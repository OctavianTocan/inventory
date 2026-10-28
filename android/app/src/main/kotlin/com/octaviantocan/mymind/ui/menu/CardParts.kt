package com.octaviantocan.mymind.ui.menu

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.rememberGraphicsLayer
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.PartAction
import com.octaviantocan.mymind.ui.card.PartChoice
import com.octaviantocan.mymind.ui.card.ReaderPart
import com.octaviantocan.mymind.ui.card.bylineOf
import com.octaviantocan.mymind.ui.card.partChoices
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import kotlinx.coroutines.launch

/**
 * The sheet a held part opens: its few rows in one raised group.
 *
 * @param part The part that was held.
 * @param card The card it belongs to.
 * @param onChoice Carries out a row.
 */
@Composable
internal fun PartContent(part: ReaderPart, card: Card, onChoice: (PartChoice) -> Unit) {
    val choices = partChoices(part, card)
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 30.dp)) {
        CardMenuGroup {
            choices.forEachIndexed { index, choice ->
                if (index > 0) SheetRowDivider()
                SheetRow(painterResource(partActionIcon(choice.action)), choice.label, { onChoice(choice) })
            }
        }
    }
}

@DrawableRes
private fun partActionIcon(action: PartAction): Int = when (action) {
    PartAction.Copy, PartAction.CopyPicture -> R.drawable.copytoclipboardicon
    PartAction.Share, PartAction.SharePicture -> R.drawable.ic_share
    PartAction.EditTags -> R.drawable.addtagsicon
    PartAction.EditNote -> R.drawable.notes
    PartAction.Read -> R.drawable.readermodeicon
    PartAction.SavePicture -> R.drawable.downloadicon
}

/**
 * Your note: the user's note on the card in a field that grows with it, and Done, which saves it. An emptied field
 * takes the note off.
 *
 * @param initial The note as it is.
 * @param onSave Called with the field's words when Done is tapped.
 * @param focusOnOpen True puts the cursor in the field when the page opens.
 */
@Composable
internal fun NotePage(initial: String, onSave: (String) -> Unit, focusOnOpen: Boolean = true) {
    val colors = menuColors()
    var text by rememberSaveable { mutableStateOf(initial) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { if (focusOnOpen) focus.requestFocus() }
    val style = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont)
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 30.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        CardMenuGroup {
            BasicTextField(
                value = text,
                onValueChange = { text = it },
                textStyle = style,
                cursorBrush = SolidColor(colors.rowText),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences),
                modifier = Modifier.fillMaxWidth().heightIn(min = 160.dp).focusRequester(focus).semantics { contentDescription = "Your note" },
                decorationBox = { field ->
                    Box(Modifier.padding(16.dp)) {
                        if (text.isEmpty()) Text("Write a note", style = style.copy(color = colors.rowText.copy(alpha = 0.5f)))
                        field()
                    }
                },
            )
        }
        SheetCta("Done", null, onClick = { onSave(text.trim()) })
    }
}

/**
 * The card as an image: a small card of its picture, title and byline, with the user's note and its tags when chosen,
 * and Share image, which hands that drawing to another app.
 *
 * @param card Card being shared.
 * @param onShare Called with the drawn card.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun CardImagePage(card: Card, onShare: (ImageBitmap) -> Unit) {
    val note = card.note?.trim()?.takeIf { it.isNotEmpty() }
    val tags = card.tags.orEmpty().mapNotNull { it.name?.trim()?.takeIf(String::isNotEmpty) }
    var withNote by rememberSaveable { mutableStateOf(note != null) }
    var withTags by rememberSaveable { mutableStateOf(false) }
    val layer = rememberGraphicsLayer()
    val scope = rememberCoroutineScope()
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 30.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.drawWithContent {
            layer.record { this@drawWithContent.drawContent() }
            drawLayer(layer)
        }.semantics { contentDescription = "The card as an image" }) {
            CardImage(card, note.takeIf { withNote }, tags.takeIf { withTags }.orEmpty())
        }
        if (note != null || tags.isNotEmpty()) {
            FlowRow(Modifier.padding(top = 18.dp), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally)) {
                if (note != null) TogglePill("Your note", withNote) { withNote = !withNote }
                if (tags.isNotEmpty()) TogglePill("Tags", withTags) { withTags = !withTags }
            }
        }
        SheetCta("Share image", R.drawable.ic_share, Modifier.padding(top = 16.dp)) { scope.launch { onShare(layer.toImageBitmap()) } }
    }
}

/** The drawing [CardImagePage] shares: the picture, title, byline, then the note and tags it was given. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun CardImage(card: Card, note: String?, tags: List<String>) {
    val colors = menuColors()
    val muted = colors.rowText.copy(alpha = 0.65f)
    Column(Modifier.width(268.dp).clip(RoundedCornerShape(24.dp)).background(colors.row).padding(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 14.dp)) {
        if (galleryPicture(card) != null) CardThumbnail(card, Modifier.fillMaxWidth().aspectRatio(3f / 2f).clip(RoundedCornerShape(18.dp)))
        Text(cardHeading(card), Modifier.padding(start = 4.dp, end = 4.dp, top = 10.dp), style = TextStyle(color = colors.rowText, fontSize = 22.sp,
            lineHeight = 28.sp, fontWeight = FontWeight.SemiBold, fontFamily = CardMenuFont), maxLines = 3, overflow = TextOverflow.Ellipsis)
        bylineOf(card)?.let {
            Text(it, Modifier.padding(start = 4.dp, end = 4.dp, top = 2.dp), style = TextStyle(color = muted, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont))
        }
        if (note != null) Text(note, Modifier.padding(start = 4.dp, end = 4.dp, top = 12.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
            .background(colors.sheet).padding(horizontal = 12.dp, vertical = 10.dp),
            style = TextStyle(color = colors.rowText, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont), maxLines = 8, overflow = TextOverflow.Ellipsis)
        if (tags.isNotEmpty()) FlowRow(Modifier.padding(start = 4.dp, end = 4.dp, top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp)) {
            tags.take(CARD_IMAGE_TAGS).forEach {
                Text(it, Modifier.heightIn(min = 28.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.08f)).padding(horizontal = 10.dp, vertical = 5.dp),
                    style = TextStyle(color = colors.rowText, fontSize = 13.sp, lineHeight = 18.sp, fontFamily = CardMenuFont))
            }
        }
    }
}

/** A quiet pill with a ring that fills while its choice is on. */
@Composable
private fun TogglePill(label: String, on: Boolean, onToggle: () -> Unit) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    Row(
        Modifier.heightIn(min = 44.dp).clip(CircleShape).background(colors.row)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Checkbox,
                onClickLabel = if (on) "Leave out" else "Include") { haptic.tick(); onToggle() }
            .semantics { this.selected = on }
            .padding(start = 12.dp, end = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SpaceRing(colors.rowText, on, 18.dp)
        Text(label, style = TextStyle(color = colors.rowText, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont))
    }
}

/**
 * A sheet's main action: a 52dp pill in the sheet's ink across the page.
 *
 * @param label What it does.
 * @param icon Artwork before the label, or null.
 * @param modifier Modifier for the pill.
 * @param enabled False dims it and ignores taps.
 * @param onClick Called on a tap.
 */
@Composable
internal fun SheetCta(label: String, @DrawableRes icon: Int?, modifier: Modifier = Modifier, enabled: Boolean = true, onClick: () -> Unit) {
    val colors = menuColors()
    Row(
        modifier.fillMaxWidth().height(52.dp).clip(CircleShape).background(colors.rowText.copy(alpha = if (enabled) 1f else 0.4f))
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) Image(painterResource(icon), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.sheet))
        Text(label, style = TextStyle(color = colors.sheet, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium, fontFamily = CardMenuFont))
    }
}

private const val CARD_IMAGE_TAGS = 4
