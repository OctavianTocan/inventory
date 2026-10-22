package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

private val ROW_SHAPE = RoundedCornerShape(10.dp)
private val BUTTON_SHAPE = RoundedCornerShape(6.dp)
private val BLUR_RADIUS = 5.dp
private const val TINT_ALPHA_ON_LIGHT = 0.35f
private const val TINT_ALPHA_ON_DARK = 0.55f

/**
 * A 48 dp frosted row with a label and an optional trailing control.
 *
 * @param label Text of the row.
 * @param backdrop Picture the row blurs.
 * @param onClick Called when the row is tapped.
 * @param modifier Modifier for the row.
 * @param trailing Control drawn after the label.
 */
@Composable
fun GlassRow(
    label: String,
    backdrop: GlassBackdrop,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    trailing: @Composable () -> Unit = {},
) {
    val colors = MymindTheme.colors
    val track = colors.settingsTrack
    val tintAlpha = if (track.luminance() >= 0.5f) TINT_ALPHA_ON_LIGHT else TINT_ALPHA_ON_DARK
    Row(
        modifier
            .fillMaxWidth()
            .height(48.dp)
            .glass(backdrop, ROW_SHAPE, background = track, tint = track.copy(alpha = tintAlpha), blurRadius = BLUR_RADIUS)
            .clickable(onClick = onClick)
            .padding(start = 16.dp, end = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            label,
            Modifier.weight(1f),
            style = TextStyle(
                color = colors.settingsRowLabel,
                fontSize = 16.sp,
                fontFamily = MymindTheme.fonts.body, fontWeight = FontWeight.Medium,
                letterSpacing = (-0.01).em,
                textAlign = TextAlign.Start,
            ),
        )
        trailing()
    }
}

/**
 * Switch with the settings drawer's colours.
 *
 * @param checked Whether the switch is on.
 * @param onCheckedChange Called with the new state when the user flips the switch.
 */
@Composable
fun SettingsSwitch(checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    val colors = MymindTheme.colors
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        colors = SwitchDefaults.colors(
            checkedThumbColor = colors.switchCheckedThumb,
            checkedTrackColor = colors.switchCheckedTrack,
            checkedBorderColor = colors.switchCheckedBorder,
            uncheckedThumbColor = colors.switchUncheckedThumb,
            uncheckedTrackColor = colors.switchUncheckedTrack,
            uncheckedBorderColor = colors.switchUncheckedBorder,
        ),
    )
}

/**
 * Icon at the end of a [GlassRow], tinted like an unchecked switch thumb and followed by an 8 dp gap.
 *
 * @param painter Icon to draw.
 */
@Composable
fun RowIcon(painter: Painter) {
    Image(painter, contentDescription = null, colorFilter = ColorFilter.tint(MymindTheme.colors.switchUncheckedThumb, BlendMode.SrcIn))
    Spacer(Modifier.width(8.dp))
}

/**
 * Red block with the caption and the account deletion button.
 *
 * @param onClick Called when the block is tapped.
 * @param modifier Modifier for the block.
 */
@Composable
fun DangerZone(onClick: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MymindTheme.colors
    val fonts = MymindTheme.fonts
    Column(
        modifier.clip(ROW_SHAPE).background(colors.danger, ROW_SHAPE).clickable(onClick = onClick).padding(6.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        BasicText(
            "DANGER ZONE",
            style = TextStyle(color = colors.dangerLabel, fontSize = 11.sp, fontFamily = fonts.body, fontWeight = FontWeight.Medium, letterSpacing = 0.48.em, lineHeight = 15.sp),
        )
        BasicText(
            "DELETE MY ACCOUNT",
            Modifier.fillMaxWidth().height(48.dp).clip(BUTTON_SHAPE).background(colors.dangerButton, BUTTON_SHAPE).wrapContentSize(),
            style = TextStyle(color = Color.White, fontSize = 14.sp, fontFamily = fonts.body, fontWeight = FontWeight.Bold, letterSpacing = 0.1.em, textAlign = TextAlign.Center),
        )
    }
}

/**
 * Version line above the logo, centred.
 *
 * @param version Version text appended to the product name.
 * @param badge Logo drawn 24 dp tall.
 * @param modifier Modifier for the footer.
 */
@Composable
fun SettingsFooter(version: String, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        BasicText(
            "mymind® v$version",
            style = TextStyle(color = MymindTheme.colors.settingsHint, fontSize = 12.sp, fontFamily = MymindTheme.fonts.body, textAlign = TextAlign.Center),
        )
    }
}
