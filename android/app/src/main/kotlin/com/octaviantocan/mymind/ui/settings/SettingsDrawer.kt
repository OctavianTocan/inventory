package com.octaviantocan.mymind.ui.settings

import android.content.Context
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.DrawerState
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val SHEET_WIDTH_FRACTION = 0.9f
private const val SCRIM_ALPHA = 0.66f

/**
 * Hosts the settings drawer over [content], filled from [settings]. The drawer opens through [drawerState] only and
 * closes by swipe, back or a tap on the scrim.
 *
 * @param drawerState State that opens and closes the drawer.
 * @param settings State and actions behind the drawer.
 * @param content Screen the drawer slides over.
 */
@Composable
fun SettingsDrawerHost(drawerState: DrawerState, settings: SettingsViewModel, content: @Composable () -> Unit) {
    val state by settings.state.collectAsStateWithLifecycle()
    LaunchedEffect(drawerState.isOpen) {
        if (drawerState.isOpen) settings.refresh()
    }
    SettingsDrawer(
        drawerState = drawerState,
        account = state.email,
        version = appVersion(LocalContext.current),
        onSignOut = settings::signOut,
        content = content,
    )
}

/**
 * The settings drawer over [content], without a view model.
 *
 * @param drawerState State that opens and closes the drawer.
 * @param account Email address of the signed-in account, or null while it is unknown.
 * @param version Version text shown in the footer.
 * @param onSignOut Called when the user taps the sign out row.
 * @param pictures Artwork of the drawer.
 * @param scrollState Scroll position of the drawer body.
 * @param content Screen the drawer slides over.
 */
@Composable
fun SettingsDrawer(
    drawerState: DrawerState,
    account: String?,
    version: String,
    onSignOut: () -> Unit,
    pictures: SettingsPictures = defaultSettingsPictures(),
    scrollState: ScrollState = rememberScrollState(),
    content: @Composable () -> Unit,
) {
    ModalNavigationDrawer(
        drawerContent = {
            ModalDrawerSheet(
                drawerState,
                Modifier.fillMaxWidth(SHEET_WIDTH_FRACTION).fillMaxHeight(),
                drawerContainerColor = MymindTheme.colors.settingsSheet,
            ) {
                SettingsSheet(account = account, version = version, onSignOut = onSignOut, pictures = pictures, scrollState = scrollState)
            }
        },
        drawerState = drawerState,
        gesturesEnabled = drawerState.isOpen,
        scrimColor = Color.Black.copy(alpha = SCRIM_ALPHA),
        content = content,
    )
}

private fun appVersion(context: Context): String {
    val info = context.packageManager.getPackageInfo(context.packageName, 0)
    return "${info.versionName} (${info.longVersionCode})"
}
