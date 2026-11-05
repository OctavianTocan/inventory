package com.octaviantocan.mymind.ui.theme

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * How the window ThyMind draws in is shaped, which decides where its pages sit (Pully's rules 206 to 209, ported). A
 * phone held upright is the shape everything was designed at; the other two show the same pages, placed differently.
 */
enum class WindowShape {
    /** Narrower than [WIDE_FROM]: one page at a time across the whole width, as on a phone held upright or split screen. */
    Phone,

    /**
     * Wide but not tall, a phone held sideways: one page at a time in a column no wider than [ReadableWidth], centred,
     * a reader and its sheets no wider than that column, and sheets that rest at their tall height because half the
     * window is too little to read.
     */
    Sideways,

    /**
     * Wide and tall, an unfolded foldable or a tablet: the gallery on the left and the card it opened beside it, each
     * about a phone's width, the gallery with its own bar.
     */
    Unfolded,
}

/** From this width a window is wide: [WindowShape.Sideways] or [WindowShape.Unfolded]. */
val WIDE_FROM: Dp = 600.dp

/** A wide window this tall or taller holds the gallery and a card side by side. */
val TALL_FROM: Dp = 600.dp

/** The widest a page's column, a reader or a sheet grows to; wider windows centre it. */
val ReadableWidth: Dp = 600.dp

/**
 * The shape of a window [width] by [height].
 *
 * @param width The window's width.
 * @param height The window's height.
 * @return [WindowShape.Phone] under [WIDE_FROM] wide, else [WindowShape.Sideways] under [TALL_FROM] tall, else
 *   [WindowShape.Unfolded].
 */
fun windowShape(width: Dp, height: Dp): WindowShape = when {
    width < WIDE_FROM -> WindowShape.Phone
    height < TALL_FROM -> WindowShape.Sideways
    else -> WindowShape.Unfolded
}

/**
 * How wide the gallery is beside an opened card in an [WindowShape.Unfolded] window, as Pully's list beside a pull
 * request: a little under half of it, never narrower than 340dp nor wider than 420dp; the card takes the rest.
 *
 * @param width The window's width.
 * @return The gallery pane's width.
 */
fun listPaneWidth(width: Dp): Dp = (width * 0.46f).coerceIn(340.dp, 420.dp)

/** The shape of the window this composable draws in; a sheet's or the reader's window is the whole screen, like the app's. */
@Composable
fun rememberWindowShape(): WindowShape {
    val size = LocalWindowInfo.current.containerSize
    if (size.width <= 0 || size.height <= 0) return WindowShape.Phone
    return with(LocalDensity.current) { windowShape(size.width.toDp(), size.height.toDp()) }
}

/**
 * Where a pane of an unfolded window sits in it, in px, so a sheet opened from it rises over that pane.
 *
 * @property left The pane's left edge in the window.
 * @property width The pane's width.
 */
class PaneSpan(val left: Float, val width: Float)

/** The pane a sheet belongs to, or null for a sheet centred along the foot of the whole window. */
val LocalPaneSpan = compositionLocalOf<PaneSpan?> { null }

/** The side insets every page keeps clear of: a camera cutout and a navigation bar at the side of a sideways phone. */
val SideInsets: WindowInsets
    @Composable get() = WindowInsets.displayCutout.union(WindowInsets.navigationBars).only(WindowInsetsSides.Horizontal)

/** Fills the width up to [ReadableWidth] and centres itself in anything wider. */
fun Modifier.readableWidth(): Modifier = fillMaxWidth().wrapContentWidth(Alignment.CenterHorizontally).widthIn(max = ReadableWidth).fillMaxWidth()
