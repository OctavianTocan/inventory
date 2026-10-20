package com.octaviantocan.mymind.ui.card

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.drawOutline
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineHeightStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import com.octaviantocan.mymind.R

/** Pully's paper and floating control colors, following the card's light or dark appearance. */
@Immutable
internal class CardSheetColors(val paper: Color, val raised: Color, val text: Color, val grabber: Color, val scrim: Color, val active: Color, val shadow: Color)

internal val LightCardSheetColors = CardSheetColors(Color(0xFFF4F2EF), Color.White, Color(0xFF0E333A), Color(0xFFD9D9D6), Color(0x610E333A), Color(0x1A0E333A), Color(0xFF0E333A))
internal val DarkCardSheetColors = CardSheetColors(Color(0xFF1B1B1B), Color(0xFF1D1D1D), Color(0xFFE3E3E3), Color(0xFF3A3A3A), Color(0x99000000), Color(0x1FFFFFFF), Color.Black)
internal val LocalCardSheetColors = compositionLocalOf { LightCardSheetColors }

/** Close and overflow glyphs; their 24-unit paths match Pully's Lu.close and Lu.more. */
@Immutable
class CardHeaderIcons(val close: Painter, val menu: Painter)

@Composable
fun defaultCardHeaderIcons() = CardHeaderIcons(painterResource(R.drawable.ic_sheet_close), painterResource(R.drawable.ic_sheet_more))

/** Pully's floating controls flanking a centered title that wraps to keep the full heading readable. */
@Composable
fun CardHeader(title: String, onClose: () -> Unit, modifier: Modifier = Modifier, onMenu: (() -> Unit)? = null, icons: CardHeaderIcons = defaultCardHeaderIcons(), enabled: Boolean = true) {
    val colors = LocalCardSheetColors.current
    val density = LocalDensity.current
    var firstLineHeight by remember(title, density) { mutableFloatStateOf(with(density) { 26.sp.toPx() }) }
    val lineHeight = with(density) { firstLineHeight.toDp() }
    val buttonTop = ((lineHeight - 44.dp) / 2f).coerceAtLeast(0.dp)
    val titleTop = ((44.dp - lineHeight) / 2f).coerceAtLeast(0.dp)
    // Keep room for the card body at the half-height stop, even with exceptionally long headings.
    val titleLimit = with(density) { (LocalWindowInfo.current.containerSize.height / 4f).toDp() }.coerceAtLeast(lineHeight)
    Row(modifier.fillMaxWidth().zIndex(1f).padding(horizontal = 16.dp), verticalAlignment = Alignment.Top) {
        CardHeaderButton(icons.close, "Close card", onClose, enabled, Modifier.padding(top = buttonTop))
        Box(Modifier.weight(1f).padding(start = 8.dp, end = 8.dp, top = titleTop, bottom = 4.dp)) {
            Text(
                title,
                modifier = Modifier.fillMaxWidth().heightIn(max = titleLimit).verticalScroll(rememberScrollState()),
                style = TextStyle(
                    fontFamily = FontFamily(Font(R.font.geist_semibold, FontWeight.SemiBold)),
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 18.sp,
                    lineHeight = 26.sp,
                    lineHeightStyle = LineHeightStyle(LineHeightStyle.Alignment.Center, LineHeightStyle.Trim.None),
                    textAlign = TextAlign.Center,
                ),
                color = colors.text,
                onTextLayout = { firstLineHeight = it.getLineBottom(0) - it.getLineTop(0) },
            )
        }
        if (onMenu != null) CardHeaderButton(icons.menu, "Card actions", onMenu, enabled, Modifier.padding(top = buttonTop)) else Spacer(Modifier.size(44.dp))
    }
}

/** Pully's two-layer canvas shadow and flat press tint, with no ripple or press scaling. */
@Composable
private fun CardHeaderButton(icon: Painter, label: String, onClick: () -> Unit, enabled: Boolean, modifier: Modifier = Modifier) {
    val colors = LocalCardSheetColors.current
    val tint = colors.shadow.copy(alpha = if (colors === DarkCardSheetColors) 0.08f * 2.7f else 0.08f)
    val fill = colors.raised
    val haptic = rememberSheetHaptics()
    Box(
        modifier.size(44.dp).drawBehind {
            val outline = CircleShape.createOutline(size, layoutDirection, this)
            val edge = Paint().apply { color = fill }
            edge.asFrameworkPaint().setShadowLayer(1f, 0f, 1.dp.toPx(), tint.copy(alpha = tint.alpha * 0.5f).toArgb())
            val drop = Paint().apply { color = fill }
            drop.asFrameworkPaint().setShadowLayer(((10.dp.toPx() / 2f - 0.5f) / 0.57735f).coerceAtLeast(0.5f), 0f, 2.dp.toPx(), tint.toArgb())
            drawIntoCanvas { it.drawOutline(outline, edge); it.drawOutline(outline, drop) }
        }.clip(CircleShape).background(fill).semantics { contentDescription = label }
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = enabled, role = Role.Button, onClickLabel = label, onClick = { haptic.tick(); onClick() }),
        contentAlignment = Alignment.Center,
    ) {
        Image(icon, contentDescription = null, modifier = Modifier.size(20.dp), colorFilter = ColorFilter.tint(colors.text))
    }
}
