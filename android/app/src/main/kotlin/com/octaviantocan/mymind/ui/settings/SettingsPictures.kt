package com.octaviantocan.mymind.ui.settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * The artwork of the settings drawer.
 *
 * @property background Backdrop of the drawer, cropped to fill and zoomed.
 * @property iconPile Picture on the icon picker row, scaled to the row width.
 * @property badge Logo at the foot of the drawer, drawn 24 dp tall.
 * @property hand Hand beside the scroll hint.
 * @property help Trailing icon of the help row.
 * @property signOut Trailing icon of the sign out row.
 */
@Immutable
class SettingsPictures(
    val background: Painter,
    val iconPile: Painter,
    val badge: Painter,
    val hand: Painter,
    val help: Painter,
    val signOut: Painter,
)

/**
 * @return The pictures that ship with the app resources, in the current appearance.
 */
@Composable
fun defaultSettingsPictures(): SettingsPictures {
    val dark = MymindTheme.colors.dark
    return SettingsPictures(
        background = painterResource(if (dark) R.drawable.settings_background_dark else R.drawable.settings_background_light),
        iconPile = painterResource(if (dark) R.drawable.icon_pile_dark else R.drawable.icon_pile_light),
        badge = painterResource(R.drawable.mymind_badge),
        hand = painterResource(R.drawable.ic_hand),
        help = painterResource(R.drawable.help),
        signOut = painterResource(R.drawable.sign_out),
    )
}
