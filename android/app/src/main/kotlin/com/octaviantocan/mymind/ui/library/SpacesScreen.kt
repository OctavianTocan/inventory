package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.ui.semantics.clearAndSetSemantics
import com.octaviantocan.mymind.ui.shell.FootCircleButton
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.scale
import com.octaviantocan.mymind.R
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Height of the title row of the Spaces list and of an opened space. */
val VIEW_HEADER_HEIGHT = GALLERY_CONTROLS_HEIGHT

private val SPACE_ROW_GAP = 10.dp
private val SPACE_SIDE = 16.dp
private val SPACE_COLUMN_GAP = 12.dp
private val SPACE_NAME_INSET = 50.dp
private val SPACE_BACK_INSET = 72.dp
private val SPACE_MENU_INSET = 68.dp
private val SPACE_BOTTOM_PADDING = 156.dp
private val STACK_SHADOW_LIGHT = Color(0xFF3C465A)
private const val STACK_SCALE = 0.9f

/**
 * The Spaces tab on the gallery's own paper: the list of spaces, or one space after the user opened it.
 *
 * @param viewModel Library behind the tab.
 * @param openSpace Space on show, or null for the list.
 * @param onOpenSpace Called with a tapped space, or null to go back to the list.
 * @param onOpenCard Called with a card tapped inside a space and that space's id.
 * @param onDeleteSpace Called when "Delete space" is picked in an opened space's menu.
 * @param onAddToSpace Called with the opened space when "Add to this space" is picked in its menu.
 * @param modifier Modifier for the page.
 * @param onLongPressCard Called with a card long-pressed inside a space and that space's id.
 * @param controls The gallery's fixed controls, drawn over the list of spaces.
 */
@Composable
fun SpacesScreen(
    viewModel: LibraryViewModel,
    openSpace: Space?,
    onOpenSpace: (Space?) -> Unit,
    onOpenCard: (Card, String) -> Unit,
    onDeleteSpace: (Space) -> Unit,
    onAddToSpace: (Space) -> Unit,
    modifier: Modifier = Modifier,
    onLongPressCard: ((Card, String) -> Unit)? = null,
    controls: @Composable BoxScope.() -> Unit = {},
) {
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { if (spaces.spaces.isEmpty()) viewModel.loadSpaces() }
    BackHandler(enabled = openSpace != null) { onOpenSpace(null) }
    if (openSpace == null) {
        SpaceListContent(
            spaces,
            onOpenSpace = onOpenSpace,
            onRetry = viewModel::loadSpaces,
            modifier = modifier,
            onRefresh = viewModel::refreshSpaces,
            controls = controls,
        )
    } else {
        val feed = viewModel.spaceFeed(openSpace.id)
        val state by feed.state.collectAsStateWithLifecycle()
        SpaceContent(
            openSpace,
            state,
            onBack = { onOpenSpace(null) },
            onLoadMore = feed::loadMore,
            onOpenCard = { onOpenCard(it, openSpace.id) },
            onAdd = { onAddToSpace(openSpace) },
            onDelete = { onDeleteSpace(openSpace) },
            modifier = modifier,
            onRefresh = { viewModel.refreshSpaceFeed(openSpace.id) },
            onLongPressCard = onLongPressCard?.let { press -> { card: Card -> press(card, openSpace.id) } },
        )
    }
}

/**
 * The list of spaces on the gallery paper: a title that scrolls away under the fixed controls, then two columns of
 * card stacks with each space's name. Creating a space is offered by the Add sheet rather than a floating button.
 *
 * @param state Spaces and their first cards.
 * @param onOpenSpace Called with a tapped space.
 * @param onRetry Called to read the spaces again after a failure.
 * @param modifier Modifier for the page.
 * @param onRefresh Called when the list is pulled down to refresh.
 * @param controls The gallery's fixed controls, drawn over the list.
 */
@Composable
fun SpaceListContent(
    state: SpacesState,
    onOpenSpace: (Space) -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
    onRefresh: () -> Unit = {},
    controls: @Composable BoxScope.() -> Unit = {},
) {
    Box(modifier.fillMaxSize().background(LibraryStyle.colors.background)) {
        when {
            state.spaces.isEmpty() && state.error != null -> WebErrorScreen(onRetry = onRetry)
            state.spaces.isEmpty() && !state.loaded -> PageSpinner()
            else -> PullRefresh(state.refreshing, onRefresh, top = VIEW_HEADER_HEIGHT) {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(2),
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = SPACE_SIDE, end = SPACE_SIDE, bottom = SPACE_BOTTOM_PADDING),
                    horizontalArrangement = Arrangement.spacedBy(SPACE_COLUMN_GAP),
                    verticalArrangement = Arrangement.spacedBy(SPACE_ROW_GAP),
                ) {
                    item(key = "spaces-title", span = { GridItemSpan(maxLineSpan) }) { GalleryTitle("All Spaces", start = 20.dp - SPACE_SIDE, end = CONTROLS_WIDTH - SPACE_SIDE) }
                    items(state.spaces, key = { it.id }) { space -> SpaceStack(space, state.covers[space.id]) { onOpenSpace(space) } }
                }
            }
        }
        controls()
    }
}

/**
 * An opened space in the gallery's photo-style mosaic, with floating Back and options controls over a title that
 * scrolls with its cards.
 *
 * @param space The space.
 * @param feed Its cards.
 * @param onBack Called when Back is tapped.
 * @param onLoadMore Called when the mosaic nears its end.
 * @param onOpenCard Called with a tapped card.
 * @param onAdd Called when "Add to this space" is picked in the options.
 * @param modifier Modifier for the page.
 * @param onDelete Called when "Delete space" is picked in the options; null leaves only the add option.
 * @param onRefresh Called when the mosaic is pulled down to refresh.
 * @param onLongPressCard Called with a long-pressed card, or null to ignore long presses.
 */
@Composable
fun SpaceContent(
    space: Space,
    feed: FeedState,
    onBack: () -> Unit,
    onLoadMore: () -> Unit,
    onOpenCard: (Card) -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    onDelete: (() -> Unit)? = null,
    onRefresh: () -> Unit = {},
    onLongPressCard: ((Card) -> Unit)? = null,
) {
    Box(modifier.fillMaxSize().background(LibraryStyle.colors.background)) {
        if (feed.empty) {
            PageSpinner()
        } else {
            PullRefresh(feed.refreshing, onRefresh, top = VIEW_HEADER_HEIGHT) {
                LibraryMosaic(feed, onLoadMore, onOpenCard, onLongPressCard = onLongPressCard, header = {
                    GalleryTitle(space.name.orEmpty(), start = SPACE_BACK_INSET, end = SPACE_MENU_INSET)
                })
            }
        }
        FootCircleButton(R.drawable.ic_sheet_back, "Back", onBack, Modifier.align(Alignment.TopStart).padding(start = 16.dp, top = 12.dp), size = 44.dp)
        SpaceOptions(onAdd, onDelete, Modifier.align(Alignment.TopEnd).padding(end = 16.dp, top = 12.dp))
    }
}

/** The opened space's options: adding to it, and deleting it, with the long-press haptic the web gives the menu. */
@Composable
private fun SpaceOptions(onAdd: () -> Unit, onDelete: (() -> Unit)?, modifier: Modifier = Modifier) {
    val haptic = LocalHapticFeedback.current
    var expanded by remember { mutableStateOf(false) }
    Box(modifier) {
        FootCircleButton(R.drawable.ic_sheet_more, "Space options", {
            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
            expanded = true
        }, size = 44.dp)
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("Add to this space") }, onClick = { expanded = false; onAdd() })
            if (onDelete != null) DropdownMenuItem(text = { Text("Delete space") }, onClick = { expanded = false; onDelete() })
        }
    }
}

/** One space (`mymind-stack`): its first card over three tilted blank cards, then the space's ring and name. */
@Composable
private fun SpaceStack(space: Space, cover: Card?, onClick: () -> Unit) {
    val colors = LibraryStyle.colors
    val shape = RoundedCornerShape(TILE_RADIUS)
    val shadows = stackShadows(MymindTheme.colors.dark)
    val angles = remember(space.id) { stackAngles(space.id) }
    BoxWithConstraints(Modifier.fillMaxWidth().clickable(interactionSource = null, indication = null, onClick = onClick)) {
        val nameWidth = maxWidth - SPACE_NAME_INSET
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Box(Modifier.fillMaxWidth().scale(STACK_SCALE)) {
                angles.asReversed().forEach { angle ->
                    Box(Modifier.matchParentSize().rotate(angle).stackShadow(shape, shadows).background(colors.raised, shape))
                }
                Box(Modifier.fillMaxWidth().aspectRatio(1f).stackShadow(shape, shadows).clip(shape).background(colors.raised)) {
                    if (cover != null) GalleryTile(cover, onClick = {}, modifier = Modifier.clearAndSetSemantics {}, enabled = false, playVideo = false)
                }
            }
            Row(Modifier.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                SpaceRing(parseHexColor(space.color) ?: colors.muted)
                Spacer(Modifier.width(10.dp))
                Text(
                    space.name.orEmpty(),
                    Modifier.widthIn(max = nameWidth),
                    style = LibraryStyle.body,
                    color = colors.text,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

/** Turns of the three blank cards under a space's first card, seeded by the space id as the web page does. */
private fun stackAngles(id: String): List<Float> {
    var angle = 0
    return (0 until 3).map { i ->
        val step = (seededUnit("${id}d$i") * 10).toInt() + 1
        angle += if (i % 2 == 0) step else -step
        angle.toFloat()
    }
}

/** A number in [0, 1) that depends only on [seed]: FNV-1a over the UTF-16 units, then two mixing rounds. */
private fun seededUnit(seed: String): Double {
    var h = -2128831035
    for (unit in seed) {
        h = h xor unit.code
        h *= 16777619
    }
    h = h xor (h ushr 13)
    h *= 0x5bd1e995
    h = h xor (h ushr 15)
    return h.toUInt().toDouble() / 4294967296.0
}

/** The two layers of `--stack-shadow` in the page's appearance (`app.css` light, `dark.css` dark). */
private fun stackShadows(dark: Boolean): List<Shadow> = if (dark) {
    listOf(
        Shadow(radius = 6.dp, color = Color.Black, offset = DpOffset(0.dp, 2.dp), alpha = 0.35f),
        Shadow(radius = 30.dp, color = Color.Black, offset = DpOffset(0.dp, 12.dp), alpha = 0.45f),
    )
} else {
    listOf(
        Shadow(radius = 5.dp, color = STACK_SHADOW_LIGHT, offset = DpOffset(0.dp, 2.dp), alpha = 0.07f),
        Shadow(radius = 28.dp, color = STACK_SHADOW_LIGHT, offset = DpOffset(0.dp, 12.dp), alpha = 0.13f),
    )
}

/** Draws [shadows] behind the content, the first one on top. */
private fun Modifier.stackShadow(shape: RoundedCornerShape, shadows: List<Shadow>) =
    shadows.asReversed().fold(this) { modifier, shadow -> modifier.dropShadow(shape, shadow) }

/** The 14 dp ring in the space's colour (`mm-circle`, radius 7, stroke 3) in its 8 dp layout box. */
@Composable
private fun SpaceRing(color: Color) {
    Canvas(Modifier.size(8.dp)) {
        val stroke = 3.dp.toPx()
        drawCircle(color, radius = 7.dp.toPx() - stroke / 2, style = Stroke(stroke))
    }
}
