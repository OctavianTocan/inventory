package com.octaviantocan.mymind.ui.shell

import androidx.annotation.DrawableRes
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.indication
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.delay
import kotlin.math.roundToInt

/** What the empty search field says, and its name for accessibility services. */
const val SEARCH_HINT = "Search your inventory"

private val NAVIGATION_INSET = 8.dp
private val TAB_SIDE = 13.dp
private val TAB_ICON = 22.dp
private val TAB_GAP = 4.dp
private val NAME_GAP = 8.dp
private const val NAME_HOLD_MILLIS = 1500L
private val LIBRARY_TABS = listOf(Tab.Everything, Tab.Spaces)

/**
 * The gallery's navigation in the one foot pill: the Everything and Spaces icon tabs inside a pill only as wide as they
 * are, and Search in the circle beside it, the two centred together. Outside a [FootBarLayer], as in a standalone
 * preview, it draws in place.
 */
@Composable
fun MymindTabBar(
    selected: Tab,
    onSelect: (Tab) -> Unit,
    modifier: Modifier = Modifier,
    onSearch: () -> Unit = {},
) {
    FootPill(
        key = "library-navigation",
        modifier = modifier,
        hug = true,
        inset = NAVIGATION_INSET,
        field = false,
        circle = FootCircle(R.drawable.ic_library_search_spark, "Search", onSearch, iconSize = 22.dp),
    ) { GalleryTabs(selected, onSelect) }
}

/**
 * The library's one bar: the navigation, or while [searching] the search field in the same pill, widened to the full
 * width, its tabs blurring into the field and the circle's Search into Close, as Pully's pages share theirs.
 *
 * @param onSubmit Called when the keyboard's search key is pressed, after the keyboard hides.
 * @param focusOnOpen True when opening search puts the cursor in its field; a swipe onto Ask leaves the keyboard down.
 * @param hint What the empty field says, such as "Search in walking" while a filter is on.
 */
@Composable
fun LibraryFootBar(
    selected: Tab,
    onSelect: (Tab) -> Unit,
    searching: Boolean,
    query: String,
    onQueryChange: (String) -> Unit,
    onSearch: () -> Unit,
    onCloseSearch: () -> Unit,
    modifier: Modifier = Modifier,
    onSubmit: () -> Unit = {},
    focusOnOpen: Boolean = true,
    hint: String = SEARCH_HINT,
) {
    if (!searching) {
        MymindTabBar(selected, onSelect, modifier, onSearch)
        return
    }
    FootPill(
        key = "library-search",
        modifier = modifier,
        search = true,
        button = query.isNotEmpty(),
        circle = FootCircle(R.drawable.ic_sheet_close, "Close search", onCloseSearch),
    ) {
        val colors = LibraryStyle.colors
        val focus = LocalFootFocus.current
        val keyboard = LocalSoftwareKeyboardController.current
        LaunchedEffect(focus) {
            if (!focusOnOpen) return@LaunchedEffect
            withFrameNanos { }
            focus?.requestFocus()
        }
        BasicTextField(
            value = query,
            onValueChange = onQueryChange,
            modifier = Modifier.weight(1f).fillMaxHeight()
                .then(if (focus != null) Modifier.focusRequester(focus) else Modifier)
                .semantics { contentDescription = SEARCH_HINT },
            singleLine = true,
            textStyle = LibraryStyle.body.copy(color = colors.text),
            cursorBrush = SolidColor(colors.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            keyboardActions = KeyboardActions(onSearch = { keyboard?.hide(); onSubmit() }),
            decorationBox = { field ->
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.CenterStart) {
                    if (query.isEmpty()) Text(hint, style = LibraryStyle.body, color = colors.muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    field()
                }
            },
        )
        if (query.isNotEmpty()) {
            FootPressTheme {
                Box(
                    Modifier.size(44.dp).clip(CircleShape).semantics { contentDescription = "Clear search" }
                        .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { onQueryChange("") },
                    contentAlignment = Alignment.Center,
                ) { FootIcon(R.drawable.ic_sheet_close, tint = colors.muted, size = 20.dp) }
            }
        }
    }
}

/**
 * Version 03 of the Everything bar (Octavian's pick, 1 November 2026): two icon tabs, the chosen one filled on its
 * highlight. Changing the tab shows the chosen place's name beside its icon for a moment, then folds it away; arriving on
 * a page shows no name. The names stay in the tabs for accessibility services and tests while folded.
 */
@Composable
private fun RowScope.GalleryTabs(selected: Tab, onSelect: (Tab) -> Unit) {
    val moving = rememberFootMotionEnabled()
    var named by remember { mutableStateOf<Tab?>(null) }
    val shown = remember { arrayOf(selected) }
    LaunchedEffect(selected) {
        if (shown[0] == selected) return@LaunchedEffect
        shown[0] = selected
        named = selected
        delay(NAME_HOLD_MILLIS)
        named = null
    }
    FootPressTheme {
        Row(
            Modifier.weight(1f).fillMaxHeight().selectableGroup(),
            horizontalArrangement = Arrangement.spacedBy(TAB_GAP, Alignment.CenterHorizontally),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            LIBRARY_TABS.forEach { tab -> GalleryTab(tab, tab == selected, tab == named, moving) { onSelect(tab) } }
        }
    }
}

/** One icon tab: the full pill height to tap, a 40dp highlight behind its icon, and its name folding out beside it while [named]. */
@Composable
private fun GalleryTab(tab: Tab, chosen: Boolean, named: Boolean, moving: Boolean, onClick: () -> Unit) {
    val colors = LibraryStyle.colors
    val haptic = LocalHapticFeedback.current
    val press = remember { MutableInteractionSource() }
    val fill by animateColorAsState(
        if (chosen) colors.selected else colors.selected.copy(alpha = 0f),
        if (moving) tween(FootMotion.FAST, easing = FootMotion.SmoothOut) else snap(),
        label = "tab-highlight",
    )
    val open = animateFloatAsState(if (named) 1f else 0f, if (moving) tween(FootMotion.MEDIUM, easing = FootMotion.SmoothOut) else snap(), label = "tab-name")
    val seen = animateFloatAsState(if (named) 1f else 0f, if (moving) tween(FootMotion.FAST, easing = FootMotion.SmoothOut) else snap(), label = "tab-name-alpha")
    Box(
        Modifier.fillMaxHeight()
            .selectable(selected = chosen, interactionSource = press, indication = null, role = Role.Tab) {
                haptic.performHapticFeedback(HapticFeedbackType.Confirm)
                onClick()
            },
        contentAlignment = Alignment.Center,
    ) {
        Row(
            Modifier.padding(vertical = NAVIGATION_INSET).fillMaxHeight()
                .clip(CircleShape).background(fill).indication(press, CardSheetIndication).padding(horizontal = TAB_SIDE),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            FootIcon(
                if (chosen) tab.filledIcon() else tab.outlineIcon(),
                tint = if (chosen) colors.text else lerp(colors.muted, colors.text, 0.45f),
                size = TAB_ICON,
            )
            Text(
                tab.label,
                Modifier.folded(open, seen, NAME_GAP),
                style = LibraryStyle.body.copy(fontSize = 15.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium),
                color = colors.text,
                maxLines = 1,
                softWrap = false,
            )
        }
    }
}

/** Shows as much of a name as [open] says, after a [gap] that opens with it, faded in by [seen]; clipped, never squeezed. */
private fun Modifier.folded(open: State<Float>, seen: State<Float>, gap: Dp): Modifier = clipToBounds().layout { measurable, constraints ->
    val placeable = measurable.measure(constraints.copy(minWidth = 0, maxWidth = Constraints.Infinity))
    val fraction = open.value
    val gapPx = gap.roundToPx()
    layout(((gapPx + placeable.width) * fraction).roundToInt(), placeable.height) {
        placeable.placeWithLayer((gapPx * fraction).roundToInt(), 0) { alpha = seen.value }
    }
}

@DrawableRes
private fun Tab.filledIcon(): Int = if (this == Tab.Everything) R.drawable.ic_library_grid_fill else R.drawable.ic_library_spaces_fill

@DrawableRes
private fun Tab.outlineIcon(): Int = if (this == Tab.Everything) R.drawable.ic_library_grid else R.drawable.ic_library_spaces
