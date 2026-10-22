package com.octaviantocan.mymind.ui.update

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.menu.LightMenuColors
import com.octaviantocan.mymind.ui.menu.MenuCard
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** @return The card colours of the current appearance. */
@Composable
private fun colors() = if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors

/**
 * The content of the request to let the app install updates.
 *
 * @param model The release that waits for the permission.
 * @param onAllow Called when "Allow mymind to install updates" is tapped.
 */
@Composable
fun AllowRequestContent(model: AllowRequestModel, onAllow: () -> Unit) {
    val colors = colors()
    MenuCard {
        Column(Modifier.padding(24.dp)) {
            Text("mymind ${model.release.versionName} is ready", style = TextStyle(color = colors.rowText, fontSize = 20.sp, fontWeight = FontWeight.Bold, fontFamily = MymindTheme.fonts.body))
            Spacer(Modifier.height(12.dp))
            Text(
                "Allow installs once, and updates install by themselves when you leave the app.",
                style = TextStyle(color = colors.rowText, fontSize = 15.sp, fontFamily = MymindTheme.fonts.body),
            )
            Spacer(Modifier.height(16.dp))
            InstallPermissionRow(onAllow)
        }
    }
}

/**
 * The row that sends the user to Android's "install unknown apps" screen for this app.
 *
 * @param onClick Called when the row is tapped.
 */
@Composable
fun InstallPermissionRow(onClick: () -> Unit) {
    val colors = colors()
    Box(
        Modifier
            .fillMaxWidth()
            .height(52.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(colors.row)
            .clickable(onClick = onClick)
            .padding(horizontal = 24.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        Text("Allow mymind to install updates", style = TextStyle(color = colors.rowText, fontSize = 17.sp, fontFamily = MymindTheme.fonts.body))
    }
}

/**
 * The request to allow installs, over the app.
 *
 * @param model The release that waits for the permission.
 * @param onLater Called when the sheet is swiped away.
 * @param onAllow Called when the install permission row is tapped.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AllowRequestSheet(model: AllowRequestModel, onLater: () -> Unit, onAllow: () -> Unit) {
    ModalBottomSheet(
        onDismissRequest = onLater,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = Color.Transparent,
        tonalElevation = 0.dp,
        dragHandle = null,
        contentWindowInsets = { WindowInsets(0, 0, 0, 0) },
    ) { AllowRequestContent(model, onAllow) }
}

/**
 * The sheet shown once after the app was updated.
 *
 * @param entry The notes of the version now running.
 * @param onClose Called when the sheet is closed.
 */
@Composable
fun AfterUpdateSheet(entry: ChangelogEntry, onClose: () -> Unit) {
    ChangelogScreen(listOf(entry), onClose)
}

/**
 * Everything the update feature draws over the signed-in app: the allow request, the after-update sheet and the
 * changelog sheet.
 *
 * @param viewModel State and actions of the update screens.
 * @param onAllow Called when the user asks to allow installs.
 */
@Composable
fun UpdateOverlays(viewModel: UpdateViewModel, onAllow: () -> Unit) {
    val sheet = viewModel.sheet.collectAsStateWithLifecycle()
    val after = viewModel.afterUpdate.collectAsStateWithLifecycle()
    val changelog = viewModel.changelogOpen.collectAsStateWithLifecycle()
    if (changelog.value) ChangelogScreen(viewModel.entries, viewModel::closeChangelog)
    sheet.value?.let { AllowRequestSheet(it, viewModel::later, onAllow) }
    if (sheet.value == null) after.value?.let { AfterUpdateSheet(it, viewModel::dismissAfterUpdate) }
}
