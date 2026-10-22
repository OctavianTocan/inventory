package com.octaviantocan.mymind.ui.settings

import android.content.Context
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.ImageLoader
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.library.rememberPublicAccountPhotoLoader
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.CardMenuHeading
import com.octaviantocan.mymind.ui.menu.PullySheet
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowChevron
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors

/** Label of the row that signs the account out. */
const val SIGN_OUT = "Sign out"

/**
 * The profile sheet filled from [settings], which it refreshes each time it opens.
 *
 * @param settings State and actions behind the sheet.
 * @param updates Update rows of the sheet.
 * @param onClose Called once the sheet has closed.
 */
@Composable
fun ProfileSheetHost(settings: SettingsViewModel, updates: SettingsUpdates, onClose: () -> Unit) {
    val state by settings.state.collectAsStateWithLifecycle()
    LaunchedEffect(settings) { settings.refresh() }
    ProfileSheet(state, appVersion(LocalContext.current), updates, onSignOut = settings::signOut, onClose = onClose,
        photoLoader = rememberPublicAccountPhotoLoader())
}

/**
 * The account and app options in Pully's bottom sheet: the Google photo, name and email, then the update rows and Sign
 * out.
 *
 * @param account The signed-in account; its fields are null until the server answers.
 * @param version Installed version, such as `0.6.0 (11)`.
 * @param updates Update rows; nothing about updates shows while [SettingsUpdates.visible] is false.
 * @param onSignOut Called when Sign out is tapped.
 * @param onClose Called once the sheet has closed.
 * @param photoLoader Loader for the account photo that never carries the library session, or null for the glyph.
 */
@Composable
fun ProfileSheet(
    account: SettingsUiState,
    version: String,
    updates: SettingsUpdates,
    onSignOut: () -> Unit,
    onClose: () -> Unit,
    photoLoader: ImageLoader? = null,
) {
    PullySheet("Profile", onClose) { ProfileSheetContent(account, version, updates, onSignOut, onClose, photoLoader) }
}

/** The body of [ProfileSheet], also drawn by screenshot fixtures inside a bare panel. */
@Composable
internal fun ColumnScope.ProfileSheetContent(
    account: SettingsUiState,
    version: String,
    updates: SettingsUpdates,
    onSignOut: () -> Unit,
    onClose: () -> Unit,
    photoLoader: ImageLoader?,
) {
    AccountCard(account, photoLoader)
    if (updates.visible) {
        CardMenuHeading("App")
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(painterResource(R.drawable.ic_tab_serendipity), "What's new", { onClose(); updates.onChangelog() },
                detail = version, trailing = { SheetRowChevron() })
            SheetRowDivider()
            SheetRow(painterResource(R.drawable.ic_check_updates), "Check for updates", updates.onCheck,
                detail = updates.status.ifEmpty { null })
            if (updates.showAllow) {
                SheetRowDivider()
                SheetRow(painterResource(R.drawable.ic_allow_installs), "Allow installs", updates.onAllow,
                    detail = "Let mymind install its own updates")
            }
        }
    }
    CardMenuHeading("Account")
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        SheetRow(painterResource(R.drawable.sign_out), SIGN_OUT, onSignOut, detail = account.email?.let { "Leave $it on this phone" })
    }
}

/** The raised card at the top: the round Google photo, or the profile glyph, beside the name and email. */
@Composable
private fun AccountCard(account: SettingsUiState, photoLoader: ImageLoader?) {
    val colors = menuColors()
    val name = account.displayName ?: account.email ?: "Your account"
    CardMenuGroup(Modifier.padding(top = 20.dp)) {
        Row(Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            Box(Modifier.size(52.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.1f))
                .clearAndSetSemantics { contentDescription = "Account photo" }, contentAlignment = Alignment.Center) {
                Image(painterResource(R.drawable.ic_library_profile), null, Modifier.size(28.dp), colorFilter = ColorFilter.tint(colors.rowText))
                if (account.photoUrl != null && photoLoader != null) {
                    AsyncImage(model = account.photoUrl, imageLoader = photoLoader, contentDescription = null,
                        modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
                }
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(name, maxLines = 1, overflow = TextOverflow.Ellipsis,
                    style = TextStyle(color = colors.rowText, fontSize = 19.sp, lineHeight = 26.sp, fontFamily = CardMenuFont))
                if (account.email != null && account.email != name) Text(account.email, maxLines = 1, overflow = TextOverflow.Ellipsis,
                    style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 14.sp, lineHeight = 20.sp, fontFamily = CardMenuFont))
            }
        }
    }
}

private fun appVersion(context: Context): String {
    val info = context.packageManager.getPackageInfo(context.packageName, 0)
    return "${info.versionName} (${info.longVersionCode})"
}
