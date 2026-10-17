package com.octaviantocan.mymind.ui.menu

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.library.parseHexColor
import com.octaviantocan.mymind.ui.theme.MymindTheme
import java.util.Locale

/**
 * Colours of the Add tags and Select a space pages in one system appearance.
 *
 * @property title Header title and suggestion text.
 * @property close Tint of the close icon.
 * @property fieldText Text and cursor of the tag field.
 * @property field Fill of the tag field.
 * @property chevron Tint of the field's chevron.
 * @property recentLabel Colour of "RECENT TAGS".
 * @property help Colour of the help and empty texts.
 * @property button Fill of the bottom button.
 * @property chip Fill of an added tag.
 * @property chipText Text of an added tag.
 * @property spaceRow Fill of a space row.
 * @property spaceLabel Name of a space row.
 */
@Immutable
class SubmenuColors(
    val title: Color,
    val close: Color,
    val fieldText: Color,
    val field: Color,
    val chevron: Color,
    val recentLabel: Color,
    val help: Color,
    val button: Color,
    val chip: Color,
    val chipText: Color,
    val spaceRow: Color,
    val spaceLabel: Color,
)

private val Slate = Color(0xFF748297)
private val Orange = Color(0xFFFF5924)

/** Submenu colours when the system is dark. */
val DarkSubmenuColors = SubmenuColors(
    title = Slate,
    close = Slate,
    fieldText = Color(0xFFA7B4C6),
    field = Color(0xFF1C1E26),
    chevron = Color(0x99748297),
    recentLabel = Color(0xFFA7B4C6),
    help = Color(0x99748297),
    button = Orange,
    chip = Color(0xFF1C1E26),
    chipText = Slate,
    spaceRow = Color(0xFF1C1E26),
    spaceLabel = Slate,
)

/** Submenu colours when the system is light. */
val LightSubmenuColors = SubmenuColors(
    title = Slate,
    close = Slate,
    fieldText = Color.Black,
    field = Color(0xFFF4F6F9),
    chevron = Color(0x4D415777),
    recentLabel = Color(0xFFB3BCCA),
    help = Color(0x9951647D),
    button = Orange,
    chip = Color(0xFFF4F6F9),
    chipText = Color.Black,
    spaceRow = Color(0xFFF4F6F9),
    spaceLabel = Color(0xFF353643),
)

@Composable
private fun submenuColors() = if (MymindTheme.colors.dark) DarkSubmenuColors else LightSubmenuColors

private const val SUGGESTION_LIMIT = 5
private const val ADD_TAGS_HELP = "Hit next to add separate tags.\nTap on added tags to remove them."
private const val NO_SPACES_HELP = "It looks like you don't have any spaces to\nadd this card to unfortunately."

/**
 * The tags the Add tags page offers under its field, as the original app picks them.
 *
 * @param all Every tag of the library, in the server's order.
 * @param recent Tags added last.
 * @param text Text in the field.
 * @return With text, the first five tags that contain it ignoring case; without, the recent tags; sorted by name
 *   ignoring case.
 */
fun tagChoices(all: List<String>, recent: List<String>, text: String): List<String> {
    val pool = if (text.isEmpty()) recent else all.filter { it.contains(text, ignoreCase = true) }.take(SUGGESTION_LIMIT)
    return pool.sortedBy { it.lowercase(Locale.ROOT) }
}

/**
 * Adds [text] to [tags] the way the Add tags page does: trimmed, without a leading `#`, and only once.
 *
 * @param tags Tags added so far.
 * @param text Typed or picked tag.
 * @return The new list.
 */
fun addTag(tags: List<String>, text: String): List<String> {
    val name = text.trim().removePrefix("#").trim()
    return if (name.isEmpty() || tags.any { it.equals(name, ignoreCase = true) }) tags else tags + name
}

/**
 * The header of a submenu: the title in capitals and the close icon.
 *
 * @param title Title.
 * @param onClose Called when the close icon is tapped.
 */
@Composable
fun SubmenuHeader(title: String, onClose: () -> Unit) {
    val colors = submenuColors()
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(top = 20.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            title.uppercase(Locale.ROOT),
            Modifier.weight(1f),
            style = TextStyle(
                color = colors.title,
                fontSize = 14.sp,
                lineHeight = 21.sp,
                letterSpacing = 0.05.em,
                fontFamily = MymindTheme.fonts.nunitoBold,
            ),
        )
        Icon(
            painterResource(R.drawable.close),
            contentDescription = "Close",
            tint = colors.close,
            modifier = Modifier
                .size(24.dp)
                .clickable(interactionSource = null, indication = ripple(bounded = false), onClick = onClose),
        )
    }
}

/**
 * The Add tags page: the tag field with its suggestions, the added tags and "Finish & Save".
 *
 * @param allTags Every tag of the library.
 * @param recentTags Tags added last.
 * @param onClose Called when the close icon is tapped.
 * @param onSave Called with the tags to add once "Finish & Save" is tapped, including text left in the field.
 * @param initialText Text the field starts with.
 * @param initialTags Tags already added when the page opens.
 * @param initiallyExpanded True to start with the suggestions open.
 * @param focusOnOpen True moves focus into the field when the page opens.
 */
@Composable
fun AddTagsPage(
    allTags: List<String>,
    recentTags: List<String>,
    onClose: () -> Unit,
    onSave: (List<String>) -> Unit,
    initialText: String = "",
    initialTags: List<String> = emptyList(),
    initiallyExpanded: Boolean = false,
    focusOnOpen: Boolean = true,
) {
    val colors = submenuColors()
    var text by remember { mutableStateOf(initialText) }
    var expanded by remember { mutableStateOf(initiallyExpanded) }
    var added by remember { mutableStateOf(initialTags) }
    val focus = remember { FocusRequester() }
    val rotation by animateFloatAsState(if (expanded) 180f else 0f, label = "chevron")
    val choices = remember(text, allTags, recentTags) { tagChoices(allTags, recentTags, text) }
    LaunchedEffect(text) { if (text.isNotEmpty()) expanded = true }
    LaunchedEffect(Unit) { if (focusOnOpen) focus.requestFocus() }

    // With the keyboard up and the suggestions open the page can be taller than the room left above the keyboard.
    Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState())) {
        SubmenuHeader(CardSubmenu.Tags.title, onClose)
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp).background(colors.field, RoundedCornerShape(16.dp))) {
            BasicTextField(
                value = text,
                onValueChange = { text = it },
                singleLine = true,
                textStyle = TextStyle(color = colors.fieldText, fontSize = 17.sp, lineHeight = 24.sp, letterSpacing = (-0.02).em, fontFamily = MymindTheme.fonts.nunito),
                cursorBrush = SolidColor(colors.fieldText),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                keyboardActions = KeyboardActions(onNext = {
                    added = addTag(added, text)
                    text = ""
                }),
                modifier = Modifier.fillMaxWidth().focusRequester(focus),
                decorationBox = { field ->
                    Row(
                        Modifier.height(56.dp).padding(horizontal = 20.dp, vertical = 16.dp),
                        horizontalArrangement = Arrangement.spacedBy(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Box(Modifier.weight(1f)) { field() }
                        Image(
                            painterResource(R.drawable.ic_chevron_down_light),
                            contentDescription = if (expanded) "Hide suggestions" else "Show suggestions",
                            colorFilter = ColorFilter.tint(colors.chevron),
                            modifier = Modifier
                                .size(16.dp)
                                .rotate(rotation)
                                .clickable(interactionSource = null, indication = null) { expanded = !expanded },
                        )
                    }
                },
            )
            AnimatedVisibility(expanded && choices.isNotEmpty(), Modifier.fillMaxWidth(), enter = expandVertically(), exit = shrinkVertically()) {
                Column {
                    HorizontalDivider(thickness = 1.dp, color = (if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors).sheet)
                    Column(Modifier.padding(vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        if (text.isEmpty()) {
                            Text(
                                "RECENT TAGS",
                                Modifier.padding(top = 8.dp).padding(horizontal = 20.dp),
                                style = TextStyle(color = colors.recentLabel, fontSize = 12.sp, lineHeight = 26.sp, fontFamily = MymindTheme.fonts.nunitoSemiBold),
                            )
                        }
                        choices.forEach { choice ->
                            Box(
                                Modifier
                                    .fillMaxWidth()
                                    .height(40.dp)
                                    .clickable {
                                        added = addTag(added, choice)
                                        expanded = false
                                        text = ""
                                    }
                                    .padding(horizontal = 20.dp),
                                contentAlignment = Alignment.CenterStart,
                            ) {
                                Text(
                                    choice,
                                    style = TextStyle(color = colors.title, fontSize = 18.sp, lineHeight = 34.sp, letterSpacing = (-0.01).em, fontFamily = MymindTheme.fonts.nunito),
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                    }
                }
            }
        }
        if (added.isEmpty()) {
            HelpText(ADD_TAGS_HELP, colors)
        } else {
            LazyRow(
                Modifier.fillMaxWidth().wrapContentHeight(),
                contentPadding = PaddingValues(20.dp, 16.dp, 20.dp, 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(added, key = { it }) { tag ->
                    Box(
                        Modifier
                            .clip(RoundedCornerShape(64.dp))
                            .background(colors.chip)
                            .clickable { added = added - tag }
                            .padding(horizontal = 20.dp, vertical = 8.dp),
                    ) {
                        Text(
                            tag,
                            style = TextStyle(
                                color = colors.chipText,
                                fontSize = 17.sp,
                                lineHeight = 23.sp,
                                letterSpacing = (-0.02).em,
                                fontFamily = MymindTheme.fonts.nunito,
                                textAlign = TextAlign.Center,
                            ),
                        )
                    }
                }
            }
        }
        SubmenuButton("Finish & Save", colors) { onSave(addTag(added, text)) }
    }
}

/**
 * The Select a space page: one row per space that puts the card in or takes it out at once, or, without spaces, the
 * empty text and "Create Your First Space".
 *
 * @param spaces Spaces in the user's order.
 * @param selected Ids of the spaces the card is in.
 * @param onClose Called when the close icon is tapped.
 * @param onToggle Called with a tapped space.
 * @param onCreateSpace Called when "Create Your First Space" is tapped.
 */
@Composable
fun SelectSpacePage(
    spaces: List<Space>,
    selected: Set<String>,
    onClose: () -> Unit,
    onToggle: (Space) -> Unit,
    onCreateSpace: () -> Unit,
) {
    val colors = submenuColors()
    Column(Modifier.fillMaxWidth().animateContentSize(), horizontalAlignment = Alignment.CenterHorizontally) {
        SubmenuHeader(CardSubmenu.Spaces.title, onClose)
        if (spaces.isEmpty()) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Image(painterResource(R.drawable.ic_spaces), contentDescription = null, modifier = Modifier.size(49.dp))
                HelpText(NO_SPACES_HELP, colors)
            }
            SubmenuButton("Create Your First Space", colors, onCreateSpace)
        } else {
            LazyColumn(
                Modifier.heightIn(max = 280.dp),
                contentPadding = PaddingValues(start = 10.dp, top = 10.dp, end = 10.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(spaces, key = { it.id }) { space -> SpaceChoiceRow(space, space.id in selected, colors) { onToggle(space) } }
            }
        }
    }
}

/** One 48 dp space row: the ring that fills and shows a check when the card is in the space, then the name. */
@Composable
private fun SpaceChoiceRow(space: Space, selected: Boolean, colors: SubmenuColors, onClick: () -> Unit) {
    val ring = parseHexColor(space.color) ?: colors.spaceLabel
    val fill by animateColorAsState(if (selected) ring else Color.Transparent, label = "fill")
    val check by animateFloatAsState(if (selected) 1f else 0f, label = "check")
    val scale = remember { Animatable(1f) }
    var first by remember { mutableStateOf(true) }
    LaunchedEffect(selected) {
        if (first) {
            first = false
            return@LaunchedEffect
        }
        val bump = spring<Float>(dampingRatio = 0.75f, stiffness = 400f)
        scale.animateTo(if (selected) 1.1f else 0.9f, bump)
        scale.animateTo(1f, bump)
    }
    val shape = RoundedCornerShape(16.dp)
    Row(
        Modifier
            .fillMaxWidth()
            .height(48.dp)
            .background(colors.spaceRow, shape)
            .clip(shape)
            .clickable(onClick = onClick)
            .padding(start = 20.dp, end = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(20.dp)
                .scale(scale.value)
                .clip(CircleShape)
                .background(fill)
                .border(2.dp, ring, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Image(
                painterResource(R.drawable.spaces_checkmark),
                contentDescription = if (selected) "In this space" else null,
                colorFilter = ColorFilter.tint(Color.White),
                modifier = Modifier.alpha(check),
            )
        }
        Text(
            space.name.orEmpty(),
            style = TextStyle(color = colors.spaceLabel, fontSize = 17.sp, lineHeight = 23.sp, fontFamily = MymindTheme.fonts.nunito),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** Centred help or empty text under a submenu's content. */
@Composable
private fun HelpText(text: String, colors: SubmenuColors) {
    Text(
        text,
        Modifier.fillMaxWidth().padding(16.dp),
        style = TextStyle(
            color = colors.help,
            fontSize = 15.sp,
            lineHeight = 20.sp,
            letterSpacing = (-0.01).em,
            fontFamily = MymindTheme.fonts.nunito,
            textAlign = TextAlign.Center,
        ),
    )
}

/** The 44 dp orange pill at the bottom of a submenu. */
@Composable
private fun SubmenuButton(label: String, colors: SubmenuColors, onClick: () -> Unit) {
    Row(
        Modifier
            .padding(horizontal = 20.dp)
            .padding(top = 8.dp, bottom = 20.dp)
            .fillMaxWidth()
            .height(44.dp)
            .clip(RoundedCornerShape(100.dp))
            .background(colors.button)
            .clickable(onClick = onClick)
            .padding(horizontal = 18.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = TextStyle(color = Color.White, fontSize = 16.sp, letterSpacing = (-0.02).em, fontFamily = MymindTheme.fonts.nunito))
    }
}
