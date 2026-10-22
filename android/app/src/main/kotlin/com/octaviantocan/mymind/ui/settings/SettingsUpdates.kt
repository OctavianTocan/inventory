package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * What the settings drawer shows about app updates.
 *
 * @property visible False to leave the update rows out.
 * @property versionLabel Text of the version row, such as `Version 0.2.0 (2)`.
 * @property status Trailing text of the "Check for updates" row.
 * @property showAllow True to show the "Allow mymind to install updates" row.
 * @property onCheck Called when "Check for updates" is tapped.
 * @property onAllow Called when the allow row is tapped.
 * @property onChangelog Called when "Changelog" is tapped.
 */
data class SettingsUpdates(
    val visible: Boolean = false,
    val versionLabel: String = "",
    val status: String = "",
    val showAllow: Boolean = false,
    val onCheck: () -> Unit = {},
    val onAllow: () -> Unit = {},
    val onChangelog: () -> Unit = {},
)

/**
 * Trailing text of a [GlassRow], followed by an 8 dp gap.
 *
 * @param text The value to show; nothing is drawn when empty.
 */
@Composable
fun RowValue(text: String) {
    if (text.isEmpty()) return
    BasicText(
        text,
        Modifier.widthIn(max = 190.dp),
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
        style = TextStyle(color = MymindTheme.colors.settingsHint, fontSize = 13.sp, fontFamily = MymindTheme.fonts.body, textAlign = TextAlign.End),
    )
    Spacer(Modifier.width(8.dp))
}
