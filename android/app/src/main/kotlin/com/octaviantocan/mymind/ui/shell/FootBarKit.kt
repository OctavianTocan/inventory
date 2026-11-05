package com.octaviantocan.mymind.ui.shell

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.background
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.BlurredEdgeTreatment
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawOutline
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.CardSheetMotion
import com.octaviantocan.mymind.ui.card.LocalCardSheetColors
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Pully's shared motion values, using the existing port for its common curve and durations. */
internal object FootMotion {
    const val QUICK = 150
    const val FAST = CardSheetMotion.FAST
    const val MEDIUM = CardSheetMotion.MEDIUM
    val SmoothOut = CardSheetMotion.SmoothOut
    val BlurMedium = 3.dp
}

/** The pill and its circle, on every page: the gallery's Google Photos height rather than Pully's 52dp. */
internal val FieldHeight = 56.dp
internal val FieldInset = 16.dp
internal val FieldButtonInset = 6.dp
internal val FieldGap = 10.dp

/** Pully Kit.floatingBar at the gallery's Google Photos geometry: one position for every bar, including while the keyboard is shown. */
fun Modifier.floatingBar(): Modifier = imePadding().navigationBarsPadding().padding(start = 12.dp, end = 12.dp, bottom = 16.dp)

@Composable
internal fun rememberFootMotionEnabled(): Boolean = rememberSheetMotionEnabled()

internal fun Modifier.footBlur(radius: Dp): Modifier =
    if (radius > 0.dp) blur(radius, BlurredEdgeTreatment.Unbounded) else this

/** Pully's downward canvas shadow levels used by the ported bar and circular controls. */
internal enum class FootShadow(val dy: Dp, val blur: Dp, val alpha: Float) {
    Button(2.dp, 10.dp, 0.08f),
    Search(2.dp, 16.dp, 0.12f),
    Menu(4.dp, 20.dp, 0.14f),
}

/** The two-layer Paint shadow from Pully Kit.floatShadow, with the gallery's own palette. */
@Composable
internal fun Modifier.footShadow(shape: Shape, level: FootShadow = FootShadow.Button): Modifier {
    val colors = LibraryStyle.colors
    val tint = colors.shadow.copy(alpha = if (MymindTheme.colors.dark) (level.alpha * 2.7f).coerceAtMost(0.9f) else level.alpha)
    val fill = colors.raised
    return drawWithCache {
        val radius = ((level.blur.toPx() / 2f - 0.5f) / 0.57735f).coerceAtLeast(0.5f)
        val broad = Paint().apply { color = fill }
        val edge = Paint().apply { color = fill }
        val outline = shape.createOutline(size, layoutDirection, this)
        edge.asFrameworkPaint().setShadowLayer(1f, 0f, 1.dp.toPx(), tint.copy(alpha = tint.alpha * 0.5f).toArgb())
        broad.asFrameworkPaint().setShadowLayer(radius, 0f, level.dy.toPx(), tint.toArgb())
        onDrawBehind {
            drawIntoCanvas {
                it.drawOutline(outline, edge)
                it.drawOutline(outline, broad)
            }
        }
    }
}

/** The published vector resource, kept out of screen-specific drawing code. */
@Composable
internal fun FootIcon(@DrawableRes id: Int, modifier: Modifier = Modifier, tint: Color = LibraryStyle.colors.text, size: Dp = 22.dp) {
    Image(painterResource(id), contentDescription = null, modifier = modifier.size(size), colorFilter = ColorFilter.tint(tint))
}

/** Reuses the native Pully press-tint port with the active gallery appearance. */
@Composable
internal fun FootPressTheme(content: @Composable () -> Unit) {
    CompositionLocalProvider(LocalCardSheetColors provides MymindTheme.palette.sheet, content = content)
}

@Composable
internal fun FootCircleButton(
    @DrawableRes icon: Int,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    size: Dp = 52.dp,
    iconSize: Dp = 22.dp,
) {
    val colors = LibraryStyle.colors
    FootPressTheme {
        Box(
            modifier.size(size).footShadow(CircleShape, FootShadow.Search).clip(CircleShape).background(colors.raised)
                .semantics { contentDescription = label }
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onClick),
            contentAlignment = Alignment.Center,
        ) { FootIcon(icon, size = iconSize) }
    }
}
