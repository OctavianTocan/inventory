package com.octaviantocan.mymind.ui.library

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
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Height of the title bar of the Spaces list and of an opened space (`.view-header`, mobile). */
val VIEW_HEADER_HEIGHT = 60.dp

private val VIEW_HEADER_GAP = 20.dp
private val SPACE_LIST_GAP = 25.dp
private val SPACE_ROW_GAP = 10.dp
private val SPACE_NAME_INSET = 50.dp
private val SPACE_NAME_COLOR = Color(0xFF40516A)
private val STACK_SHADOW_LIGHT = Color(0xFF3C465A)
private val FAB_END = 15.dp
private val FAB_BOTTOM = 50.dp
private val FAB_SHADOW = Shadow(radius = 22.dp, color = Color.Black, offset = DpOffset(5.dp, 5.dp), alpha = 0.25f)
private const val STACK_SCALE = 0.9f

/**
 * The Spaces tab: the list of spaces, or one space after the user opened it.
 *
 * @param viewModel Library behind the tab.
 * @param openSpace Space on show, or null for the list.
 * @param onOpenSpace Called with a tapped space, or null to go back to the list.
 * @param onOpenCard Called with a card tapped inside a space and that space's id.
 * @param onNewSpace Called when the + button of the list is tapped.
 * @param onDeleteSpace Called when "Delete space" is picked in an opened space's menu.
 * @param modifier Modifier for the page.
 */
@Composable
fun SpacesScreen(
    viewModel: LibraryViewModel,
    openSpace: Space?,
    onOpenSpace: (Space?) -> Unit,
    onOpenCard: (Card, String) -> Unit,
    onNewSpace: () -> Unit,
    onDeleteSpace: (Space) -> Unit,
    modifier: Modifier = Modifier,
) {
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { if (spaces.spaces.isEmpty()) viewModel.loadSpaces() }
    BackHandler(enabled = openSpace != null) { onOpenSpace(null) }
    if (openSpace == null) {
        SpaceListContent(spaces, onOpenSpace = onOpenSpace, onRetry = viewModel::loadSpaces, onAdd = onNewSpace, modifier = modifier)
    } else {
        val feed = viewModel.spaceFeed(openSpace.id)
        val state by feed.state.collectAsStateWithLifecycle()
        SpaceContent(
            openSpace,
            state,
            onBack = { onOpenSpace(null) },
            onLoadMore = feed::loadMore,
            onOpenCard = { onOpenCard(it, openSpace.id) },
            onAdd = {},
            onDelete = { onDeleteSpace(openSpace) },
            modifier = modifier,
        )
    }
}

/**
 * The list of spaces: a centred title, then two columns of card stacks with each space's name.
 *
 * @param state Spaces and their first cards.
 * @param onOpenSpace Called with a tapped space.
 * @param onRetry Called to read the spaces again after a failure.
 * @param onAdd Called when the + button is tapped.
 * @param modifier Modifier for the page.
 */
@Composable
fun SpaceListContent(state: SpacesState, onOpenSpace: (Space) -> Unit, onRetry: () -> Unit, onAdd: () -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize().background(MymindTheme.page.background)) {
        when {
            state.spaces.isEmpty() && state.error != null -> WebErrorScreen(onRetry = onRetry)
            state.spaces.isEmpty() && !state.loaded -> PageSpinner()
            else -> LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(start = PAGE_GUTTER, end = PAGE_GUTTER, top = VIEW_HEADER_HEIGHT + SPACE_LIST_GAP, bottom = GRID_BOTTOM_PADDING),
                horizontalArrangement = Arrangement.spacedBy(PAGE_GUTTER),
                verticalArrangement = Arrangement.spacedBy(SPACE_ROW_GAP),
            ) {
                items(state.spaces, key = { it.id }) { space -> SpaceStack(space, state.covers[space.id]) { onOpenSpace(space) } }
            }
        }
        ViewHeader(title = "All Spaces")
        AddFab(onAdd, Modifier.align(Alignment.BottomEnd))
    }
}

/**
 * An opened space: back arrow, the space's name and a menu button over the space's card grid.
 *
 * @param space The space.
 * @param feed Its cards.
 * @param onBack Called when the back arrow is tapped.
 * @param onLoadMore Called when the grid nears its end.
 * @param onOpenCard Called with a tapped card.
 * @param onAdd Called when the + button is tapped.
 * @param modifier Modifier for the page.
 * @param onDelete Called when "Delete space" is picked in the menu; null leaves the menu dots without a menu.
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
) {
    Box(modifier.fillMaxSize().background(MymindTheme.page.background)) {
        if (feed.empty) PageSpinner() else CardGrid(feed, onLoadMore, onOpenCard, top = VIEW_HEADER_HEIGHT + VIEW_HEADER_GAP)
        ViewHeader(title = space.name.orEmpty(), onBack = onBack, showMenu = true, onDelete = onDelete)
        AddFab(onAdd, Modifier.align(Alignment.BottomEnd))
    }
}

/** The 60 dp title bar: optional back arrow, centred title, optional menu dots that open "Delete space". */
@Composable
private fun ViewHeader(title: String, onBack: (() -> Unit)? = null, showMenu: Boolean = false, onDelete: (() -> Unit)? = null) {
    val color = viewHeaderColor()
    Box(Modifier.fillMaxWidth().height(VIEW_HEADER_HEIGHT).background(MymindTheme.page.background)) {
        Text(
            title,
            Modifier.align(Alignment.Center).padding(horizontal = 60.dp),
            style = TextStyle(
                color = color,
                fontSize = 19.sp,
                lineHeight = 26.sp,
                letterSpacing = (-0.03).em,
                fontFamily = MymindTheme.fonts.nunito,
                textAlign = TextAlign.Center,
            ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (onBack != null) {
            Canvas(
                Modifier
                    .align(Alignment.CenterStart)
                    .padding(start = 21.dp)
                    .size(16.dp)
                    .semantics { contentDescription = "Back" }
                    .clickable(interactionSource = null, indication = null, role = Role.Button, onClick = onBack),
            ) {
                val chevron = Path().apply {
                    moveTo(10.98.dp.toPx(), 1.06.dp.toPx())
                    lineTo(4.58.dp.toPx(), 7.46.dp.toPx())
                    lineTo(10.98.dp.toPx(), 13.86.dp.toPx())
                }
                drawPath(chevron, color, style = Stroke(2.11.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round))
            }
        }
        if (showMenu) {
            var expanded by remember { mutableStateOf(false) }
            Box(Modifier.align(Alignment.CenterEnd)) {
                val tap = if (onDelete != null) {
                    Modifier
                        .semantics { contentDescription = "Space options" }
                        .clickable(interactionSource = null, indication = null, role = Role.Button) { expanded = true }
                } else {
                    Modifier
                }
                Canvas(Modifier.size(60.dp).then(tap)) {
                    val r = 2.dp.toPx()
                    val gap = 6.dp.toPx()
                    for (i in -1..1) drawCircle(color.copy(alpha = 0.7f), r, Offset(center.x + i * gap, center.y))
                }
                if (onDelete != null) {
                    DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                        DropdownMenuItem(
                            text = { Text("Delete space") },
                            onClick = {
                                expanded = false
                                onDelete()
                            },
                        )
                    }
                }
            }
        }
    }
}

/** Colour of view header titles (`.view-header --color`). */
@Composable
private fun viewHeaderColor(): Color = if (MymindTheme.colors.dark) Color(0xFF626C7A) else Color(0xFF383F4A)

/** One space (`mymind-stack`): its first card over three tilted blank cards, then the space's ring and name. */
@Composable
private fun SpaceStack(space: Space, cover: Card?, onClick: () -> Unit) {
    val page = MymindTheme.page
    val shape = RoundedCornerShape(TILE_RADIUS)
    val shadows = stackShadows(MymindTheme.colors.dark)
    val angles = remember(space.id) { stackAngles(space.id) }
    BoxWithConstraints(Modifier.fillMaxWidth().clickable(interactionSource = null, indication = null, onClick = onClick)) {
        val nameWidth = maxWidth - SPACE_NAME_INSET
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Box(Modifier.fillMaxWidth().scale(STACK_SCALE)) {
                angles.asReversed().forEach { angle ->
                    Box(Modifier.matchParentSize().rotate(angle).stackShadow(shape, shadows).background(page.surface, shape))
                }
                Box(Modifier.stackShadow(shape, shadows).clip(shape)) {
                    if (cover != null) StackCover(cover) else Spacer(Modifier.fillMaxWidth().aspectRatio(1f).background(page.surface))
                }
            }
            Row(Modifier.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                SpaceRing(parseHexColor(space.color) ?: page.muted)
                Spacer(Modifier.width(10.dp))
                Text(
                    space.name.orEmpty(),
                    Modifier.widthIn(max = nameWidth),
                    style = TextStyle(
                        color = SPACE_NAME_COLOR,
                        fontSize = 17.sp,
                        lineHeight = 20.sp,
                        letterSpacing = (-0.02).em,
                        fontFamily = MymindTheme.fonts.nunito,
                    ),
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

/** The front card of a space stack: the card's tile at its grid height, without its caption. */
@Composable
private fun StackCover(card: Card) {
    when (tileKind(card)) {
        TileKind.Media, TileKind.Video -> CardPicture(card, play = tileKind(card) == TileKind.Video)
        else -> CardTile(card, onClick = {}, caption = false)
    }
}

/** The 14 dp ring in the space's colour (`mm-circle`, radius 7, stroke 3) in its 8 dp layout box. */
@Composable
private fun SpaceRing(color: Color) {
    Canvas(Modifier.size(8.dp)) {
        val stroke = 3.dp.toPx()
        drawCircle(color, radius = 7.dp.toPx() - stroke / 2, style = Stroke(stroke))
    }
}

/**
 * The orange + button at the bottom right of the Spaces pages, 50 dp above the end of the page.
 *
 * @param onClick Called when tapped.
 * @param modifier Modifier that places the button.
 */
@Composable
fun AddFab(onClick: () -> Unit, modifier: Modifier = Modifier) {
    val square = with(LocalDensity.current) { fabSquare() }
    AddButton(
        onClick,
        modifier.padding(end = FAB_END, bottom = FAB_BOTTOM).dropShadow(RoundedCornerShape(6.dp), FAB_SHADOW),
        plus = 22.dp,
        square = square,
    )
}

/** The button's exact square inside its laid-out box: padding rounds to whole pixels, the web page does not. */
private fun Density.fabSquare(): Rect {
    val box = SEARCH_BAR_HEIGHT.roundToPx().toFloat()
    val right = box + FAB_END.roundToPx() - FAB_END.toPx()
    val bottom = box + FAB_BOTTOM.roundToPx() - FAB_BOTTOM.toPx()
    val size = SEARCH_BAR_HEIGHT.toPx()
    return Rect(right - size, bottom - size, right, bottom)
}
