package com.octaviantocan.mymind.ui.update

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
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

private val Accent = Color(0xFFFF7A00)

/** @return The card colours of the current appearance. */
@Composable
private fun colors() = if (MymindTheme.colors.dark) DarkMenuColors else LightMenuColors

/** A heading and the lines of one version. */
@Composable
private fun Notes(title: String, lines: List<String>) {
    val colors = colors()
    Text(title, style = TextStyle(color = colors.rowText, fontSize = 20.sp, fontWeight = FontWeight.Bold, fontFamily = MymindTheme.fonts.nunito))
    Spacer(Modifier.height(12.dp))
    lines.forEach {
        Text(
            "•  $it",
            Modifier.padding(bottom = 6.dp),
            style = TextStyle(color = colors.rowText, fontSize = 15.sp, fontFamily = MymindTheme.fonts.nunito),
        )
    }
}

/** A full-width pill button. */
@Composable
private fun PillButton(label: String, primary: Boolean, modifier: Modifier = Modifier, onClick: () -> Unit) {
    val colors = colors()
    Box(
        modifier
            .height(48.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(if (primary) Accent else colors.row)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            label,
            style = TextStyle(color = if (primary) Color.White else colors.rowText, fontSize = 16.sp, fontWeight = FontWeight.Bold, fontFamily = MymindTheme.fonts.nunito),
        )
    }
}

/**
 * The content of the "What's new" sheet for an available update.
 *
 * @param model What to announce and how far the update is.
 * @param onInstall Called when Install is tapped.
 * @param onLater Called when Later is tapped.
 * @param onAllow Called when "Allow mymind to install updates" is tapped.
 * @param onRetry Called when Try again is tapped after a failure.
 */
@Composable
fun UpdateSheetContent(model: UpdateSheetModel, onInstall: () -> Unit, onLater: () -> Unit, onAllow: () -> Unit, onRetry: () -> Unit) {
    val colors = colors()
    MenuCard {
        Column(Modifier.padding(24.dp)) {
            Notes("What's new in ${model.release.versionName}", model.release.notes)
            Spacer(Modifier.height(8.dp))
            when (model.phase) {
                UpdatePhase.Downloading -> {
                    LinearProgressIndicator(progress = { model.progress }, Modifier.fillMaxWidth(), color = Accent, trackColor = colors.row)
                    Spacer(Modifier.height(16.dp))
                    PillButton("Later", primary = false, Modifier.fillMaxWidth(), onLater)
                }
                UpdatePhase.Installing -> Text("Installing", style = TextStyle(color = colors.rowText, fontSize = 15.sp, fontFamily = MymindTheme.fonts.nunito))
                UpdatePhase.Failed -> {
                    Text(model.failure.orEmpty(), style = TextStyle(color = colors.dangerText, fontSize = 15.sp, fontFamily = MymindTheme.fonts.nunito))
                    Spacer(Modifier.height(16.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        PillButton("Later", primary = false, Modifier.weight(1f), onLater)
                        PillButton("Try again", primary = true, Modifier.weight(1f), onRetry)
                    }
                }
                UpdatePhase.Ready -> {
                    if (!model.installAllowed) {
                        InstallPermissionRow(onAllow)
                        Spacer(Modifier.height(12.dp))
                    }
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        PillButton("Later", primary = false, Modifier.weight(1f), onLater)
                        PillButton("Install", primary = model.installAllowed, Modifier.weight(1f), onInstall)
                    }
                }
            }
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
        Text("Allow mymind to install updates", style = TextStyle(color = colors.rowText, fontSize = 17.sp, fontFamily = MymindTheme.fonts.nunito))
    }
}

/**
 * The update sheet over the app.
 *
 * @param model What to announce.
 * @param onInstall Called when Install is tapped.
 * @param onLater Called when Later is tapped or the sheet is swiped away.
 * @param onAllow Called when the install permission row is tapped.
 * @param onRetry Called when Try again is tapped.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UpdateSheet(model: UpdateSheetModel, onInstall: () -> Unit, onLater: () -> Unit, onAllow: () -> Unit, onRetry: () -> Unit) {
    ModalBottomSheet(
        onDismissRequest = onLater,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = Color.Transparent,
        tonalElevation = 0.dp,
        dragHandle = null,
        contentWindowInsets = { WindowInsets(0, 0, 0, 0) },
    ) { UpdateSheetContent(model, onInstall, onLater, onAllow, onRetry) }
}

/**
 * The content of the sheet shown once after the app was updated.
 *
 * @param entry The notes of the version now running.
 * @param onClose Called when Got it is tapped.
 */
@Composable
fun AfterUpdateContent(entry: ChangelogEntry, onClose: () -> Unit) {
    MenuCard {
        Column(Modifier.padding(24.dp)) {
            Notes("What's new in ${entry.versionName}", entry.notes)
            Spacer(Modifier.height(8.dp))
            PillButton("Got it", primary = true, Modifier.fillMaxWidth(), onClose)
        }
    }
}

/**
 * The sheet shown once after the app was updated.
 *
 * @param entry The notes of the version now running.
 * @param onClose Called when the sheet is closed.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AfterUpdateSheet(entry: ChangelogEntry, onClose: () -> Unit) {
    ModalBottomSheet(
        onDismissRequest = onClose,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = Color.Transparent,
        tonalElevation = 0.dp,
        dragHandle = null,
        contentWindowInsets = { WindowInsets(0, 0, 0, 0) },
    ) { AfterUpdateContent(entry, onClose) }
}

/**
 * The screen with every version of the bundled changelog.
 *
 * @param entries Versions, newest first.
 * @param onClose Called when Close or Back is pressed.
 */
@Composable
fun ChangelogScreen(entries: List<ChangelogEntry>, onClose: () -> Unit) {
    val colors = colors()
    BackHandler(onBack = onClose)
    Column(Modifier.fillMaxSize().background(MymindTheme.colors.page).statusBarsPadding().padding(horizontal = 24.dp)) {
        Row(Modifier.fillMaxWidth().height(56.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Changelog", Modifier.weight(1f), style = TextStyle(color = colors.rowText, fontSize = 22.sp, fontWeight = FontWeight.Bold, fontFamily = MymindTheme.fonts.nunito))
            Text("Close", Modifier.clickable(onClick = onClose).padding(8.dp), style = TextStyle(color = Accent, fontSize = 16.sp, fontFamily = MymindTheme.fonts.nunito))
        }
        Column(Modifier.verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
            entries.forEach { entry ->
                Text(
                    "${entry.versionName} (${entry.versionCode}), ${entry.date}",
                    Modifier.padding(top = 16.dp, bottom = 8.dp),
                    style = TextStyle(color = colors.rowText, fontSize = 17.sp, fontWeight = FontWeight.Bold, fontFamily = MymindTheme.fonts.nunito),
                )
                entry.notes.forEach {
                    Text("•  $it", Modifier.padding(bottom = 4.dp), style = TextStyle(color = colors.rowText, fontSize = 15.sp, fontFamily = MymindTheme.fonts.nunito))
                }
            }
        }
    }
}

/**
 * Everything the update feature draws over the signed-in app: the update sheet, the after-update sheet and the
 * changelog screen.
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
    sheet.value?.let { UpdateSheet(it, viewModel::install, viewModel::later, onAllow, viewModel::retry) }
    if (sheet.value == null) after.value?.let { AfterUpdateSheet(it, viewModel::dismissAfterUpdate) }
}
