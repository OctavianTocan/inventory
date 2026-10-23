package com.octaviantocan.mymind.ui.shell

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.indication
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onPlaced
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private val NAVIGATION_HEIGHT = 56.dp
private val NAVIGATION_GAP = 8.dp
private val NAVIGATION_INSET = 8.dp
private val TAB_SIDE = 16.dp
private val LIBRARY_TABS = listOf(Tab.Everything, Tab.Spaces)

/** Registers the gallery's navigation with Pully's one root bar, or renders in place in a standalone preview. */
@Composable
fun MymindTabBar(
    selected: Tab,
    onSelect: (Tab) -> Unit,
    modifier: Modifier = Modifier,
    onSearch: () -> Unit = {},
) {
    FootFree("library-navigation", modifier) { GalleryNavigation(selected, onSelect, onSearch) }
}

/** The shared bar switches its contents to the search field without creating a page-owned floating bar. */
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
        trailing = {
            FootCircleButton(R.drawable.ic_sheet_close, "Close search", onCloseSearch, Modifier.padding(start = NAVIGATION_GAP))
        },
    ) {
        val colors = LibraryStyle.colors
        val focus = LocalFootFocus.current
        val keyboard = LocalSoftwareKeyboardController.current
        LaunchedEffect(focus) {
            withFrameNanos { }
            focus?.requestFocus()
        }
        BasicTextField(
            value = query,
            onValueChange = onQueryChange,
            modifier = Modifier.weight(1f).fillMaxHeight()
                .then(if (focus != null) Modifier.focusRequester(focus) else Modifier)
                .semantics { contentDescription = "Search your mind" },
            singleLine = true,
            textStyle = LibraryStyle.body.copy(color = colors.text),
            cursorBrush = SolidColor(colors.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
            keyboardActions = KeyboardActions(onSearch = { keyboard?.hide() }),
            decorationBox = { field ->
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.CenterStart) {
                    if (query.isEmpty()) Text("Search your mind", style = LibraryStyle.body, color = colors.muted, maxLines = 1)
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
 * Pully CapsuleTabs' one sliding highlight, adapted to the Photos capsule and separate search circle: as in Google
 * Photos, the highlight hugs the chosen tab's icon and label, while each tab still takes half the capsule to tap.
 */
@Composable
private fun GalleryNavigation(selected: Tab, onSelect: (Tab) -> Unit, onSearch: () -> Unit) {
    val colors = LibraryStyle.colors
    val haptic = LocalHapticFeedback.current
    val moving = rememberFootMotionEnabled()
    val selectedIndex = if (selected == Tab.Spaces) 1 else 0
    val density = LocalDensity.current
    val fontScale = density.fontScale
    FootPressTheme {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(NAVIGATION_GAP)) {
            BoxWithConstraints(
                Modifier.weight(1f).height(NAVIGATION_HEIGHT).footShadow(CircleShape, FootShadow.Search)
                    .clip(CircleShape).background(colors.raised)
                    .pointerInput(Unit) { detectTapGestures { } }
                    .padding(NAVIGATION_INSET).selectableGroup(),
            ) {
                val segment = constraints.maxWidth / LIBRARY_TABS.size
                val spans = remember { mutableStateMapOf<Int, IntRange>() }
                val highlight = remember { Animatable(0f) to Animatable(0f) }
                val target = spans[selectedIndex]
                LaunchedEffect(target, moving) {
                    if (target == null) return@LaunchedEffect
                    val (x, width) = highlight
                    val first = width.value == 0f
                    if (first || !moving) {
                        x.snapTo(target.first.toFloat())
                        width.snapTo((target.last - target.first).toFloat())
                    } else coroutineScope {
                        val spec = tween<Float>(FootMotion.FAST, easing = FootMotion.SmoothOut)
                        launch { x.animateTo(target.first.toFloat(), spec) }
                        launch { width.animateTo((target.last - target.first).toFloat(), spec) }
                    }
                }
                Box(
                    Modifier.offset { IntOffset(highlight.first.value.roundToInt(), 0) }
                        .width(with(density) { highlight.second.value.toDp() }).fillMaxHeight().clip(CircleShape).background(colors.selected),
                )
                val showIcons = maxWidth >= 256.dp && fontScale <= 1.2f
                Row(Modifier.fillMaxSize()) {
                    LIBRARY_TABS.forEachIndexed { index, tab ->
                        val chosen = index == selectedIndex
                        val press = remember { MutableInteractionSource() }
                        Box(
                            Modifier.weight(1f).fillMaxHeight()
                                .selectable(selected = chosen, interactionSource = press, indication = null, role = Role.Tab) {
                                    haptic.performHapticFeedback(HapticFeedbackType.Confirm)
                                    onSelect(tab)
                                },
                            contentAlignment = if (index == 0) Alignment.CenterStart else Alignment.CenterEnd,
                        ) {
                            Row(
                                Modifier.fillMaxHeight()
                                    .onPlaced { spot ->
                                        val start = segment * index + spot.positionInParent().x.roundToInt()
                                        spans[index] = start..start + spot.size.width
                                    }
                                    .clip(CircleShape).indication(press, CardSheetIndication).padding(horizontal = TAB_SIDE),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                            ) {
                                if (showIcons && chosen) {
                                    FootIcon(if (tab == Tab.Everything) R.drawable.ic_library_grid_fill else R.drawable.ic_library_spaces_fill, size = 18.dp)
                                }
                                Text(
                                    tab.label,
                                    Modifier.weight(1f, fill = false),
                                    style = LibraryStyle.body.copy(fontSize = 15.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium),
                                    color = if (chosen) colors.text else lerp(colors.muted, colors.text, 0.45f),
                                    maxLines = 1,
                                    softWrap = false,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                    }
                }
            }
            FootCircleButton(R.drawable.ic_library_search_spark, "Search", onSearch, size = NAVIGATION_HEIGHT, iconSize = 24.dp)
        }
    }
}
