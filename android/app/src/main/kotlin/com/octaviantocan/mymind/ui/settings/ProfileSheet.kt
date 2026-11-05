package com.octaviantocan.mymind.ui.settings

import android.Manifest
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil3.ImageLoader
import coil3.compose.AsyncImage
import com.octaviantocan.aikit.AiCli
import com.octaviantocan.aikit.AiSignInServices
import com.octaviantocan.mymind.MymindApp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.feedback.FeedbackTracker
import com.octaviantocan.mymind.ui.library.rememberPublicAccountPhotoLoader
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.CardMenuHeading
import com.octaviantocan.mymind.ui.menu.PullyPagedSheet
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowChevron
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors
import com.octaviantocan.mymind.ui.theme.LocalThemePrefs
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.ThemePrefs

/** Label of the row that signs the account out. */
const val SIGN_OUT = "Sign out"

/**
 * The profile sheet filled from [settings], which it refreshes each time it opens.
 *
 * @param settings State and actions behind the sheet.
 * @param updates Update rows of the sheet.
 * @param onClose Called once the sheet has closed.
 * @param ai The account's Claude Code and Codex, or null for a sheet without the AI rows.
 * @param feedback Sends feedback, or null for a sheet without the Send feedback row.
 * @param photoLoader The gallery's account-photo loader, whose memory already holds the photo, or null for one of its own.
 * @param pageScheme The account's own appearance, which the pages follow until one is picked on the phone.
 */
@Composable
fun ProfileSheetHost(
    settings: SettingsViewModel,
    updates: SettingsUpdates,
    onClose: () -> Unit,
    ai: AiAccount? = null,
    feedback: FeedbackTracker? = null,
    photoLoader: ImageLoader? = null,
    pageScheme: PageScheme = PageScheme.System,
) {
    val state by settings.state.collectAsStateWithLifecycle()
    val agents by settings.agents.collectAsStateWithLifecycle()
    LaunchedEffect(settings) { settings.refresh() }
    ProfileSheet(state, appVersion(LocalContext.current), updates, onSignOut = settings::signOut, onClose = onClose,
        photoLoader = photoLoader ?: rememberPublicAccountPhotoLoader(), ai = ai, feedback = feedback,
        agents = AgentsActions(agents, settings::loadAgents, settings::removeAgent), themes = LocalThemePrefs.current, pageScheme = pageScheme)
}

/**
 * The account and app options in Pully's bottom sheet: the Google photo, name and email, the AI search rows, then the
 * update rows and Sign out. The AI rows open their pages inside the same sheet.
 *
 * @param account The signed-in account; its fields are null until the server answers.
 * @param version Installed version, such as `0.6.0 (11)`.
 * @param updates Update rows; nothing about updates shows while [SettingsUpdates.visible] is false.
 * @param onSignOut Called when Sign out is tapped.
 * @param onClose Called once the sheet has closed.
 * @param photoLoader Loader for the account photo that never carries the library session, or null for the glyph.
 * @param ai The account's Claude Code and Codex, or null for a sheet without the AI rows.
 * @param feedback Sends feedback, or null for a sheet without the Send feedback row and page.
 * @param agents The Connected agents page's state and actions, or null for a sheet without that row.
 * @param themes The phone's theme setting, or null for a sheet without the Appearance group.
 * @param pageScheme The account's own appearance, which the pages follow until one is picked on the phone.
 */
@Composable
fun ProfileSheet(
    account: SettingsUiState,
    version: String,
    updates: SettingsUpdates,
    onSignOut: () -> Unit,
    onClose: () -> Unit,
    photoLoader: ImageLoader? = null,
    ai: AiAccount? = null,
    feedback: FeedbackTracker? = null,
    agents: AgentsActions? = null,
    themes: ThemePrefs? = null,
    pageScheme: PageScheme = PageScheme.System,
) {
    var page by remember { mutableStateOf<ProfilePage?>(null) }
    val services = ai?.let { rememberAiSignInServices(it) }
    val context = LocalContext.current
    val notify = { text: String -> Toast.makeText(context, text, Toast.LENGTH_SHORT).show() }
    val askNotifications = rememberAskNotifications()
    PullyPagedSheet("Profile", page, onBack = { page = null }, onClose = onClose) { shown ->
        if (shown == ProfilePage.Agents && agents != null) {
            ConnectedAgentsPage(agents.state, agents.onLoad, agents.onRemove)
        } else if (shown == ProfilePage.Theme && themes != null) {
            ThemePage(themes)
        } else if (shown == null || shown == ProfilePage.Agents || shown == ProfilePage.Theme || ai == null || services == null) {
            ProfileSheetContent(account, version, updates, onSignOut, onClose, photoLoader,
                aiRows = ai?.let { mine -> { AiRows(mine, feedback = feedback != null) { page = it } } },
                onAgents = agents?.let { { page = ProfilePage.Agents } },
                appearance = themes?.let { prefs -> rememberAppearanceRows(prefs, pageScheme) { page = ProfilePage.Theme } })
        } else when (shown) {
            ProfilePage.Model -> ModelPage(ai, ai.prefs, open = { page = it }, onDone = { page = null })
            ProfilePage.Effort -> EffortPage(ai, ai.prefs, onDone = { page = null })
            ProfilePage.Feedback -> if (feedback != null) {
                FeedbackPage(ai, feedback, open = { page = it }, onProviders = { page = null }, askNotifications = askNotifications)
            }
            ProfilePage.FeedbackModel -> ModelPage(ai, ai.feedbackPrefs, open = { page = it }, onDone = { page = ProfilePage.Feedback })
            ProfilePage.FeedbackEffort -> EffortPage(ai, ai.feedbackPrefs, onDone = { page = ProfilePage.Feedback })
            ProfilePage.ClaudeSignIn -> AiSignInPage(services, AiCli.Claude, notify)
            ProfilePage.CodexSignIn -> AiSignInPage(services, AiCli.Codex, notify)
            ProfilePage.Agents, ProfilePage.Theme -> Unit
        }
    }
}

/**
 * Asks for the notification permission once, so a feedback's progress can show; does nothing when it is granted or was asked for.
 *
 * @return The call that asks, to be made as feedback is sent.
 */
@Composable
private fun rememberAskNotifications(): () -> Unit {
    val context = LocalContext.current
    val container = (context.applicationContext as? MymindApp)?.container
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
    return {
        if (container != null && !container.updateNotifier.allowed() && !container.updatePrefs.askedNotifications) {
            container.updatePrefs.askedNotifications = true
            launcher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }
}

/**
 * What the Connected agents page needs from the sheet's owner.
 *
 * @property state The connected AI apps.
 * @property onLoad Asks the server for them again.
 * @property onRemove Signs one app out by its id.
 */
class AgentsActions(val state: AgentsUiState, val onLoad: () -> Unit, val onRemove: (String) -> Unit)

/** The sign-in pages' services for [ai], reading the clipboard only while the app is in front. */
@Composable
private fun rememberAiSignInServices(ai: AiAccount): AiSignInServices {
    val context = LocalContext.current
    val uri = LocalUriHandler.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    val resumed = lifecycle.currentStateFlow.collectAsState()
    return remember(ai) {
        val clipboard = context.getSystemService(ClipboardManager::class.java)
        ai.services(
            foreground = { resumed.value.isAtLeast(Lifecycle.State.RESUMED) },
            open = { runCatching { uri.openUri(it) } },
            copy = { clipboard?.setPrimaryClip(ClipData.newPlainText("Sign-in code", it)) },
            copied = { runCatching { clipboard?.primaryClip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.coerceToText(context)?.toString() }.getOrNull() },
        )
    }
}

/** The body of [ProfileSheet]'s root page, also drawn by screenshot fixtures inside a bare panel. */
@Composable
internal fun ColumnScope.ProfileSheetContent(
    account: SettingsUiState,
    version: String,
    updates: SettingsUpdates,
    onSignOut: () -> Unit,
    onClose: () -> Unit,
    photoLoader: ImageLoader?,
    aiRows: (@Composable () -> Unit)? = null,
    onAgents: (() -> Unit)? = null,
    appearance: AppearanceRows? = null,
) {
    AccountCard(account, photoLoader)
    aiRows?.invoke()
    if (appearance != null) {
        CardMenuHeading("Appearance")
        AppearanceTabs(appearance.selected, appearance.onSelect)
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(painterResource(R.drawable.ic_palette), "Theme", appearance.onTheme, detail = appearance.themeName, trailing = { SheetRowChevron() })
        }
    }
    if (updates.visible) {
        CardMenuHeading("App")
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(painterResource(R.drawable.samevibeicon), "What's new", { onClose(); updates.onChangelog() },
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
        if (onAgents != null) {
            SheetRow(painterResource(R.drawable.ic_library_search_spark), CONNECTED_AGENTS, onAgents, trailing = { SheetRowChevron() })
            SheetRowDivider()
        }
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
