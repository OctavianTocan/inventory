package com.octaviantocan.mymind.ui.library

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** The colours the web's new space modal offers, in its order and spelling. */
val SPACE_COLORS = listOf(
    "#75ffc0", "#c0c2ce", "#ff8fa4", "#000", "#19aad1", "#17c37b", "#ffdcd0", "#166ff4",
    "#cdff06", "#f96", "#bfb5d7", "#96cbd1", "#cba0aa", "#fdf06f", "#7a30cf", "#ef3e4a",
)

private val BACKDROP = Color.Black.copy(alpha = 0.8f)
private const val INTRO = "A space is a collection of cards inside your mind. Upload directly into a space, or pick a card from the overview."
private const val COLOR_HINT = "Color coding your space helps you\nto spot it a lot easier when you need it."

/**
 * The new space modal of the web front-end, in two steps: name the space, then pick its colour.
 *
 * @param onDismiss Called when the backdrop is tapped or Back is pressed.
 * @param onCreate Called with the name and colour once "Finish & Save" is tapped.
 */
@Composable
fun NewSpaceDialog(onDismiss: () -> Unit, onCreate: (name: String, color: String) -> Unit) {
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false, decorFitsSystemWindows = false)) {
        Box(
            Modifier.fillMaxSize().background(BACKDROP).clickable(interactionSource = null, indication = null, onClick = onDismiss).imePadding(),
            contentAlignment = Alignment.Center,
        ) {
            NewSpaceContent(onCreate, Modifier.clickable(interactionSource = null, indication = null) {})
        }
    }
}

/**
 * The box of the new space modal without the dialog window.
 *
 * @param onCreate Called with the name and colour once "Finish & Save" is tapped.
 * @param modifier Modifier for the box.
 * @param initialName Name the field starts with.
 * @param initialStep 1 for the name, 2 for the colour.
 * @param initialColor Colour picked when the box opens, or null.
 * @param focusOnOpen True moves focus into the name field.
 */
@Composable
fun NewSpaceContent(
    onCreate: (name: String, color: String) -> Unit,
    modifier: Modifier = Modifier,
    initialName: String = "",
    initialStep: Int = 1,
    initialColor: String? = null,
    focusOnOpen: Boolean = true,
) {
    var name by remember { mutableStateOf(initialName) }
    var step by remember { mutableStateOf(initialStep) }
    var color by remember { mutableStateOf(initialColor) }
    val dark = MymindTheme.colors.dark
    Box(
        modifier
            .padding(horizontal = PAGE_GUTTER)
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(MymindTheme.page.surface)
            .padding(start = 30.dp, top = 40.dp, end = 30.dp, bottom = 30.dp),
    ) {
        AnimatedContent(step, label = "step") { shown ->
            Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
                if (shown == 1) {
                    Image(painterResource(R.drawable.ic_spaces), contentDescription = null, modifier = Modifier.size(57.dp))
                    Heading("Create new space", dark)
                    Paragraph(INTRO, dark)
                    NameField(name, { name = it }, dark, focusOnOpen) { if (name.isNotBlank()) step = 2 }
                    ModalButton("NEXT STEP", enabled = name.isNotBlank(), fill = Color(0xFF626C7A), disabledFill = if (dark) Color(0xFF626C7A) else Color(0xFFD3D3D3), text = if (dark) Color(0xFF1D1E28) else Color.White) { step = 2 }
                } else {
                    Heading("Pick a color", dark)
                    Paragraph(COLOR_HINT, dark)
                    ColorChooser(color) { color = it }
                    ModalButton("FINISH & SAVE", enabled = color != null, fill = MymindTheme.colors.accent, disabledFill = MymindTheme.colors.accent.copy(alpha = 0.5f), text = Color.White) {
                        color?.let { onCreate(name.trim(), it) }
                    }
                }
            }
        }
    }
}

@Composable
private fun Heading(text: String, dark: Boolean) {
    Text(
        text,
        style = TextStyle(
            color = if (dark) Color.White else Color(0xFF3E4B5F),
            fontSize = 34.sp,
            lineHeight = 1.1.em,
            letterSpacing = (-0.03).em,
            fontFamily = MymindTheme.fonts.louize,
            textAlign = TextAlign.Center,
        ),
    )
}

@Composable
private fun Paragraph(text: String, dark: Boolean) {
    Text(
        text,
        Modifier.padding(top = 10.dp, bottom = 20.dp),
        style = TextStyle(
            color = if (dark) Color(0xFFA7B4C6) else Color(0xFF697A94),
            fontSize = 19.sp,
            fontWeight = FontWeight.Light,
            letterSpacing = (-0.01).em,
            fontFamily = MymindTheme.fonts.nunito,
            textAlign = TextAlign.Center,
        ),
    )
}

/** The centred name input; IME Done goes to the colour step. */
@Composable
private fun NameField(value: String, onChange: (String) -> Unit, dark: Boolean, focusOnOpen: Boolean, onDone: () -> Unit) {
    val focus = remember { FocusRequester() }
    val ink = if (dark) Color.White else Color(0xFF748297)
    LaunchedEffect(Unit) { if (focusOnOpen) focus.requestFocus() }
    BasicTextField(
        value = value,
        onValueChange = onChange,
        singleLine = true,
        textStyle = TextStyle(color = ink, fontSize = 19.sp, fontFamily = MymindTheme.fonts.nunito, textAlign = TextAlign.Center),
        cursorBrush = SolidColor(ink),
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { onDone() }),
        modifier = Modifier.padding(top = 10.dp, bottom = 20.dp).fillMaxWidth().focusRequester(focus),
        decorationBox = { field ->
            Box(
                Modifier.fillMaxWidth().clip(RoundedCornerShape(6.dp)).background(if (dark) Color(0xFF14161E) else Color(0xFFFBFCFD)).padding(15.dp),
                contentAlignment = Alignment.Center,
            ) {
                if (value.isEmpty()) {
                    Text(
                        "Name your new space",
                        style = TextStyle(color = if (dark) Color(0xFF626C7A) else Color(0xFF748297), fontSize = 19.sp, fontFamily = MymindTheme.fonts.nunito, textAlign = TextAlign.Center),
                    )
                }
                field()
            }
        },
    )
}

/** The colour rings; the picked one is filled. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ColorChooser(selected: String?, onSelect: (String) -> Unit) {
    FlowRow(
        Modifier.fillMaxWidth().padding(bottom = 30.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp, Alignment.CenterHorizontally),
        verticalArrangement = Arrangement.spacedBy(14.dp),
        maxItemsInEachRow = 4,
    ) {
        SPACE_COLORS.forEach { hex ->
            val color = parseHexColor(hex) ?: Color.Gray
            val chosen = hex == selected
            Canvas(
                Modifier
                    .size(33.dp)
                    .semantics {
                        contentDescription = "Space color $hex"
                        this.selected = chosen
                    }
                    .clickable(interactionSource = null, indication = null, role = Role.RadioButton) { onSelect(hex) },
            ) {
                val stroke = 4.dp.toPx()
                if (chosen) drawCircle(color) else drawCircle(color, radius = size.minDimension / 2 - stroke / 2, style = Stroke(stroke))
            }
        }
    }
}

@Composable
private fun ModalButton(label: String, enabled: Boolean, fill: Color, disabledFill: Color, text: Color, onClick: () -> Unit) {
    Box(
        Modifier
            .height(40.dp)
            .clip(RoundedCornerShape(50))
            .background(if (enabled) fill else disabledFill)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 20.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = TextStyle(color = text, fontSize = 13.sp, fontFamily = MymindTheme.fonts.nunitoSemiBold))
    }
}
