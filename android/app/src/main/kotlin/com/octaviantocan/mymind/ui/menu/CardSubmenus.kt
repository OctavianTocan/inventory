package com.octaviantocan.mymind.ui.menu

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
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
import androidx.compose.ui.graphics.luminance
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
import java.util.Locale

/**
 * Colours of the Add tags and Select a space pages in one system appearance.
 *
 * @property title Header title and suggestion text.
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
 * @property buttonText Label colour that contrasts with the button fill.
 */
@Immutable
class SubmenuColors(
    val title: Color,
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
    val buttonText: Color = Color.White,
)

/** Pully surfaces and type follow the card; the primary action uses its actual accent. */
@Composable
private fun submenuColors(): SubmenuColors {
    val colors = menuColors()
    val button = LocalCardMenuAccent.current ?: colors.rowText
    return SubmenuColors(
        title = colors.rowText, fieldText = colors.rowText,
        field = colors.row, chevron = colors.rowText.copy(alpha = 0.6f),
        recentLabel = colors.rowText.copy(alpha = 0.65f), help = colors.rowText.copy(alpha = 0.7f),
        button = button, chip = colors.row, chipText = colors.rowText,
        spaceRow = colors.row, spaceLabel = colors.rowText,
        buttonText = if (button.luminance() > 0.179f) Color.Black else Color.White,
    )
}

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
 * The Add tags page: the tag field with its suggestions, the added tags and "Finish & Save".
 *
 * @param allTags Every tag of the library.
 * @param recentTags Tags added last.
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

    // The sheet owns the viewport so dragging and IME changes share one scroll owner.
    Column(Modifier.fillMaxWidth().padding(top = 16.dp)) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp).background(colors.field, RoundedCornerShape(16.dp))) {
            BasicTextField(
                value = text,
                onValueChange = { text = it },
                singleLine = true,
                textStyle = TextStyle(color = colors.fieldText, fontSize = 17.sp, lineHeight = 24.sp, letterSpacing = (-0.02).em, fontFamily = CardMenuFont),
                cursorBrush = SolidColor(colors.fieldText),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                keyboardActions = KeyboardActions(onNext = {
                    added = addTag(added, text)
                    text = ""
                }),
                modifier = Modifier.fillMaxWidth().focusRequester(focus),
                decorationBox = { field ->
                    Row(
                        Modifier.heightIn(min = 56.dp).padding(horizontal = 16.dp, vertical = 12.dp),
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
                    HorizontalDivider(thickness = 1.dp, color = menuColors().sheet)
                    Column(Modifier.padding(vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        if (text.isEmpty()) {
                            Text(
                                "RECENT TAGS",
                                Modifier.padding(top = 8.dp).padding(horizontal = 20.dp),
                                style = TextStyle(color = colors.recentLabel, fontSize = 12.sp, lineHeight = 26.sp, fontFamily = CardMenuFont),
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
                                    style = TextStyle(color = colors.title, fontSize = 18.sp, lineHeight = 34.sp, letterSpacing = (-0.01).em, fontFamily = CardMenuFont),
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
                                fontFamily = CardMenuFont,
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
 * @param onToggle Called with a tapped space.
 * @param onCreateSpace Called when "Create Your First Space" is tapped.
 */
@Composable
fun SelectSpacePage(
    spaces: List<Space>,
    selected: Set<String>,
    onToggle: (Space) -> Unit,
    onCreateSpace: () -> Unit,
) {
    val colors = submenuColors()
    Column(Modifier.fillMaxWidth().padding(top = 16.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        if (spaces.isEmpty()) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Image(painterResource(R.drawable.ic_spaces), contentDescription = null, modifier = Modifier.size(49.dp))
                HelpText(NO_SPACES_HELP, colors)
            }
            SubmenuButton("Create Your First Space", colors, onCreateSpace)
        } else {
            Column(Modifier.padding(horizontal = 16.dp).padding(bottom = 30.dp)
                .fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(colors.spaceRow)) {
                spaces.forEachIndexed { index, space ->
                    if (index > 0) HorizontalDivider(Modifier.padding(start = 52.dp), color = colors.spaceLabel.copy(alpha = 0.1f), thickness = 1.dp)
                    SpaceChoiceRow(space, space.id in selected, colors) { onToggle(space) }
                }
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
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 52.dp)
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
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
            style = TextStyle(color = colors.spaceLabel, fontSize = 17.sp, lineHeight = 23.sp, fontFamily = CardMenuFont),
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
            fontFamily = CardMenuFont,
            textAlign = TextAlign.Center,
        ),
    )
}

/** The 44 dp action pill at the bottom of a submenu. */
@Composable
private fun SubmenuButton(label: String, colors: SubmenuColors, onClick: () -> Unit) {
    Row(
        Modifier
            .padding(horizontal = 20.dp)
            .padding(top = 8.dp, bottom = 20.dp)
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clip(RoundedCornerShape(100.dp))
            .background(colors.button)
            .clickable(onClick = onClick)
            .padding(horizontal = 18.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = TextStyle(color = colors.buttonText, fontSize = 16.sp, letterSpacing = (-0.02).em, fontFamily = CardMenuFont))
    }
}
