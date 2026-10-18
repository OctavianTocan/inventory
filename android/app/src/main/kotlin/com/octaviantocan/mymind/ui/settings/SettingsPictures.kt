package com.octaviantocan.mymind.ui.settings

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import com.octaviantocan.mymind.R
/**
 * The icons of the settings drawer.
 *
 * @property hand Hand beside the scroll hint.
 * @property help Trailing icon of the help row.
 * @property signOut Trailing icon of the sign out row.
 */
@Immutable
class SettingsPictures(
    val hand: Painter,
    val help: Painter,
    val signOut: Painter,
)

/**
 * @return The icons that ship with the app resources.
 */
@Composable
fun defaultSettingsPictures(): SettingsPictures = SettingsPictures(
    hand = painterResource(R.drawable.ic_hand),
    help = painterResource(R.drawable.help),
    signOut = painterResource(R.drawable.sign_out),
)
