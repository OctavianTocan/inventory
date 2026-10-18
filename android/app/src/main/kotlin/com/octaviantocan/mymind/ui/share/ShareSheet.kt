package com.octaviantocan.mymind.ui.share

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.share.ShareDetails
import com.octaviantocan.mymind.share.ShareState
import com.octaviantocan.mymind.ui.library.UploadQueueState
import com.octaviantocan.mymind.ui.library.parseHexColor
import com.octaviantocan.mymind.ui.menu.addTag
import com.octaviantocan.mymind.ui.menu.tagChoices
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.upload.UploadQueueStrip

/** The tabs of the share sheet. */
enum class ShareTab(val label: String, val icon: Int) {
    Tags("Tags", R.drawable.tags),
    Notes("Notes", R.drawable.notes),
    Spaces("Spaces", R.drawable.spaces),
}

/** How far the save has come. */
enum class SavePhase { Editing, Saving, Saved, Failed }

/**
 * Everything the share sheet shows and lets the user change.
 *
 * @param tab Tab that is open.
 * @param allTags Every tag of the library, for suggestions.
 * @param recentTags Tags added last.
 * @param spaces The user's spaces.
 * @param phase How far the save has come.
 */
@Stable
class ShareSheetState(
    tab: ShareTab = ShareTab.Tags,
    allTags: List<String> = emptyList(),
    recentTags: List<String> = emptyList(),
    spaces: List<Space> = emptyList(),
    phase: SavePhase = SavePhase.Editing,
) {
    var tab by mutableStateOf(tab)
    var allTags by mutableStateOf(allTags)
    var recentTags by mutableStateOf(recentTags)
    var spaces by mutableStateOf(spaces)
    var phase by mutableStateOf(phase)
    var tagText by mutableStateOf("")
    var tags by mutableStateOf<List<String>>(emptyList())
    var note by mutableStateOf("")
    var selectedSpaces by mutableStateOf<Set<String>>(emptySet())
    var uploads by mutableStateOf(UploadQueueState())

    /** The details to put on the saved cards, with text left in the tag field counted as a tag. */
    fun details() = ShareDetails(addTag(tags, tagText), note, selectedSpaces)
}

/** Colours of the share sheet; mymind uses the same ones in light and dark. */
object ShareColors {
    val sheet = Color(0xFFF0F2F5)
    val tabInactive = Color(0xFF121B28)
    val tabActive = Color.White
    val field = Color.White
    val fieldText = Color.Black
    val hint = Color(0xFF747E8D)
    val label = Color(0xFF415777)
    val chip = Color(0xFFDEE4ED)
    val button = Color(0xFFFF5924)
}


/**
 * The share sheet: title, tabs, the open tab and the Save button over the picture, with the upload strip below.
 *
 * @param state What the sheet shows.
 * @param onSave Called when Save is tapped.
 * @param onToggleUploads Called when the upload strip is tapped.
 * @param onDismissUpload Called with a failed upload's id.
 */
@Composable
fun ShareSheet(
    state: ShareSheetState,
    onSave: () -> Unit,
    onToggleUploads: () -> Unit,
    onDismissUpload: (Long) -> Unit,
) {
    Box(Modifier.fillMaxSize().background(ShareColors.sheet)) {
        Column(Modifier.fillMaxSize().statusBarsPadding().imePadding().navigationBarsPadding()) {
            Text(
                titleOf(state.phase),
                Modifier.padding(horizontal = 28.dp).padding(top = 32.dp, bottom = 20.dp),
                style = TextStyle(
                    color = Color.White,
                    fontSize = 38.sp,
                    lineHeight = 36.sp,
                    letterSpacing = 0.03.em,
                    fontFamily = MymindTheme.fonts.louize,
                ),
            )
            TabRow(state)
            Column(
                Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
                    .background(ShareColors.sheet),
            ) {
                Box(Modifier.weight(1f).fillMaxWidth()) {
                    when (state.tab) {
                        ShareTab.Tags -> TagsPanel(state)
                        ShareTab.Notes -> NotePanel(state)
                        ShareTab.Spaces -> SpacesPanel(state)
                    }
                }
                if (state.phase == SavePhase.Failed) {
                    Text(
                        "Saving failed. Try again.",
                        Modifier.fillMaxWidth().padding(horizontal = 20.dp),
                        style = bodyStyle(ShareColors.label).copy(textAlign = TextAlign.Center),
                    )
                }
                SaveButton(state.phase, onSave)
            }
        }
        UploadQueueStrip(
            state.uploads,
            onToggle = onToggleUploads,
            onDismiss = onDismissUpload,
            modifier = Modifier.align(Alignment.BottomCenter),
            bottom = 80.dp,
        )
    }
}

private fun titleOf(phase: SavePhase) = if (phase == SavePhase.Saved) "Tagged and saved to your mind" else "Saving to your mind"

@Composable
private fun TabRow(state: ShareSheetState) {
    Row(Modifier.padding(horizontal = 28.dp).padding(bottom = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        ShareTab.entries.forEach { tab ->
            val color = if (tab == state.tab) ShareColors.tabActive else ShareColors.tabInactive
            Row(
                Modifier.clip(RoundedCornerShape(8.dp)).clickable { state.tab = tab }.padding(vertical = 4.dp, horizontal = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Image(painterResource(tab.icon), contentDescription = null, colorFilter = ColorFilter.tint(color))
                Text(
                    tab.label,
                    style = TextStyle(color = color, fontSize = 20.sp, lineHeight = 28.sp, letterSpacing = 0.02.em, fontFamily = MymindTheme.fonts.nunito),
                )
            }
        }
    }
}

@Composable
private fun bodyStyle(color: Color) =
    TextStyle(color = color, fontSize = 15.sp, lineHeight = 20.sp, fontFamily = MymindTheme.fonts.nunito)

@Composable
private fun TagsPanel(state: ShareSheetState) {
    val choices = tagChoices(state.allTags, state.recentTags, state.tagText)
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(ShareColors.field).padding(20.dp)) {
            if (state.tagText.isEmpty()) Text("+ Add tags", style = bodyStyle(ShareColors.hint).copy(fontSize = 15.sp, lineHeight = 20.sp))
            BasicTextField(
                value = state.tagText,
                onValueChange = { state.tagText = it },
                singleLine = true,
                textStyle = TextStyle(color = ShareColors.fieldText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.nunito),
                cursorBrush = SolidColor(ShareColors.fieldText),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                keyboardActions = KeyboardActions(onNext = {
                    state.tags = addTag(state.tags, state.tagText)
                    state.tagText = ""
                }),
                modifier = Modifier.fillMaxWidth(),
            )
        }
        if (state.tags.isEmpty()) {
            Text("Hit next to add separate tags.\nTap on added tags to remove them.", Modifier.fillMaxWidth(), style = bodyStyle(ShareColors.hint).copy(textAlign = TextAlign.Center))
        } else {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                state.tags.forEach { tag ->
                    Text(
                        tag,
                        Modifier.clip(RoundedCornerShape(64.dp)).background(ShareColors.chip).clickable { state.tags = state.tags - tag }.padding(horizontal = 16.dp, vertical = 6.dp),
                        style = bodyStyle(ShareColors.fieldText).copy(fontSize = 17.sp, lineHeight = 23.sp),
                    )
                }
            }
        }
        if (choices.isNotEmpty()) {
            if (state.tagText.isEmpty()) Text("RECENT TAGS", style = bodyStyle(ShareColors.hint).copy(fontSize = 12.sp, fontFamily = MymindTheme.fonts.nunitoSemiBold))
            choices.forEach { choice ->
                Text(
                    choice,
                    Modifier.fillMaxWidth().clickable {
                        state.tags = addTag(state.tags, choice)
                        state.tagText = ""
                    }.padding(vertical = 6.dp),
                    style = bodyStyle(ShareColors.label).copy(fontSize = 18.sp, lineHeight = 26.sp),
                )
            }
        }
    }
}

@Composable
private fun NotePanel(state: ShareSheetState) {
    Box(Modifier.fillMaxSize().padding(20.dp)) {
        Box(Modifier.fillMaxWidth().heightIn(min = 160.dp).clip(RoundedCornerShape(8.dp)).background(ShareColors.field).padding(20.dp)) {
            if (state.note.isEmpty()) Text("Write a note", style = bodyStyle(ShareColors.hint))
            BasicTextField(
                value = state.note,
                onValueChange = { state.note = it },
                textStyle = TextStyle(color = ShareColors.fieldText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.nunito),
                cursorBrush = SolidColor(ShareColors.fieldText),
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun SpacesPanel(state: ShareSheetState) {
    if (state.spaces.isEmpty()) {
        Text(
            "You have no spaces.\nPlease create a new space\nin the mymind app first.",
            Modifier.fillMaxWidth().padding(28.dp),
            style = bodyStyle(ShareColors.hint).copy(textAlign = TextAlign.Center),
        )
        return
    }
    LazyColumn(Modifier.fillMaxSize(), contentPadding = androidx.compose.foundation.layout.PaddingValues(20.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        items(state.spaces, key = { it.id }) { space ->
            val selected = space.id in state.selectedSpaces
            Row(
                Modifier
                    .fillMaxWidth()
                    .height(48.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(ShareColors.field)
                    .clickable { state.selectedSpaces = if (selected) state.selectedSpaces - space.id else state.selectedSpaces + space.id }
                    .padding(horizontal = 20.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.size(10.dp).clip(RoundedCornerShape(5.dp)).background(parseHexColor(space.color) ?: ShareColors.label))
                Text(
                    space.name.orEmpty(),
                    Modifier.weight(1f).padding(start = 12.dp),
                    maxLines = 1,
                    style = bodyStyle(ShareColors.fieldText).copy(fontSize = 18.sp, lineHeight = 32.sp),
                )
                if (selected) Image(painterResource(R.drawable.checkmark), contentDescription = "In this space", colorFilter = ColorFilter.tint(ShareColors.label))
            }
        }
    }
}

@Composable
private fun SaveButton(phase: SavePhase, onSave: () -> Unit) {
    Box(Modifier.fillMaxWidth().padding(20.dp), contentAlignment = Alignment.Center) {
        Box(
            Modifier
                .height(48.dp)
                .clip(RoundedCornerShape(100.dp))
                .background(ShareColors.button.copy(alpha = if (phase == SavePhase.Editing || phase == SavePhase.Failed) 1f else 0.5f))
                .clickable(enabled = phase == SavePhase.Editing || phase == SavePhase.Failed, onClick = onSave)
                .padding(horizontal = 48.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                when (phase) {
                    SavePhase.Saving -> "Saving"
                    SavePhase.Saved -> "Saved"
                    else -> "Save"
                },
                style = TextStyle(color = Color.White, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.nunito),
            )
        }
    }
}

/**
 * A card over the picture that explains why nothing was saved.
 *
 * @param problem The state to explain; [ShareState.Ready] shows nothing.
 * @param onAction Called when the card's button is tapped.
 * @param onClose Called when the area around the card is tapped.
 */
@Composable
fun ShareProblem(problem: ShareState, onAction: () -> Unit, onClose: () -> Unit) {
    val text = problemText(problem) ?: return
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.5f)).clickable(interactionSource = null, indication = null, onClick = onClose),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            Modifier
                .padding(horizontal = 24.dp)
                .clip(RoundedCornerShape(16.dp))
                .background(ShareColors.sheet)
                .clickable(interactionSource = null, indication = null) {}
                .padding(28.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Image(painterResource(R.drawable.sad_mymind), contentDescription = null, colorFilter = ColorFilter.tint(ShareColors.tabInactive))
            Text(
                text.title,
                style = TextStyle(color = ShareColors.tabInactive, fontSize = 45.sp, lineHeight = 44.sp, fontFamily = MymindTheme.fonts.louize, textAlign = TextAlign.Center),
            )
            Text(
                text.body,
                style = TextStyle(color = ShareColors.label, fontSize = 22.sp, lineHeight = 26.sp, fontFamily = MymindTheme.fonts.nunito, textAlign = TextAlign.Center),
            )
            Box(
                Modifier.height(48.dp).clip(RoundedCornerShape(100.dp)).background(ShareColors.button).clickable(onClick = onAction).padding(horizontal = 32.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(text.button, style = TextStyle(color = Color.White, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.nunito))
            }
        }
    }
}

/** Words of a problem card. */
class ProblemText(val title: String, val body: String, val button: String)

/**
 * The words the share target shows for a state.
 *
 * @param problem The state.
 * @return Title, body and button, or null for a state that is not a problem.
 */
fun problemText(problem: ShareState): ProblemText? = when (problem) {
    ShareState.Malformed -> ProblemText("Ouch, that hurt.", "A rare and unknown error occurred. I'm sorry.\n\nPlease screenshot this message and send it to help@mymind.com", "Okay I got it")
    ShareState.TooManyItems -> ProblemText("That's too much!", "Your mind can only upload\nup to 20 things at once.\n\nTry again, with a bit less!", "Okay I got it")
    ShareState.FileTooBig -> ProblemText("Whoa, that's big.", "mymind only supports files\nup to 25mb, which means\nthis file is reeeally big.\n\nTry a smaller one?", "Okay I got it")
    ShareState.PdfHasPassword -> ProblemText("We can't get in!", "Your mind doesn't support\nPDF files that are password protected.", "Okay I got it")
    ShareState.NoLogin -> ProblemText("Please log in!", "mymind couldn't save this\nbecause you're not logged\ninto your account.\n\nPlease sign in and\nsave it again.", "Sign in")
    is ShareState.Ready -> null
}
