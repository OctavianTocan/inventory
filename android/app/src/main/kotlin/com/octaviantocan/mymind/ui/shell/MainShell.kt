package com.octaviantocan.mymind.ui.shell

import android.content.ClipData
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDrawerState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.card.CardSheet
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.errors.ApiErrorDialog
import com.octaviantocan.mymind.ui.library.EverythingScreen
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.NewSpaceDialog
import com.octaviantocan.mymind.ui.library.SerendipityScreen
import com.octaviantocan.mymind.ui.library.SpacesScreen
import com.octaviantocan.mymind.ui.menu.AddTagsPage
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuOutcome
import com.octaviantocan.mymind.ui.menu.CardMenuSheet
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.menu.SelectSpacePage
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.ui.settings.SettingsDrawerHost
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import kotlinx.coroutines.launch

/** Distance from the bottom of the screen to the end of the library page; the tab bar covers the rest. */
val PAGE_BOTTOM_GAP = 80.dp

private val MENU_BLUR = 8.dp

/**
 * A card the user opened.
 *
 * @property card The card.
 * @property spaceId Space it was opened from, or null.
 */
data class OpenedCard(val card: Card, val spaceId: String?)

/**
 * The signed-in shell: the page of the selected tab with the tab bar over its bottom edge, an opened card over both,
 * the card menu, and the settings drawer.
 *
 * @param viewModel Library behind the tabs and the card menu.
 * @param settingsViewModel State and actions behind the settings drawer.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainShell(viewModel: LibraryViewModel, settingsViewModel: SettingsViewModel) {
    var tab by rememberSaveable { mutableStateOf(Tab.Everything) }
    var openSpace by remember { mutableStateOf<Space?>(null) }
    var opened by remember { mutableStateOf<OpenedCard?>(null) }
    var menuOpen by remember { mutableStateOf(false) }
    var submenu by remember { mutableStateOf<CardSubmenu?>(null) }
    var newSpaceOpen by remember { mutableStateOf(false) }
    var deleting by remember { mutableStateOf<Space?>(null) }
    var failure by remember { mutableStateOf<ApiFailure?>(null) }
    val scheme by viewModel.scheme.collectAsStateWithLifecycle()
    val features by viewModel.features.collectAsStateWithLifecycle()
    val tagChoices by viewModel.tagChoices.collectAsStateWithLifecycle()
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    val clipboard = LocalClipboard.current
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    val menuProgress by animateFloatAsState(if (menuOpen) 1f else 0f, label = "menu")
    val openCard = { card: Card, spaceId: String? -> opened = OpenedCard(card, spaceId) }
    val updateCard = { card: Card -> opened = opened?.takeIf { it.card.id == card.id }?.copy(card = card) ?: opened }
    val closeMenu: () -> Unit = {
        scope.launch {
            sheetState.hide()
            menuOpen = false
            submenu = null
        }
    }

    SettingsDrawerHost(drawerState, settingsViewModel) {
        ShellFrame(
            tab = tab,
            onSelectTab = { tab = it },
            scheme = scheme,
            modifier = Modifier.blur(MENU_BLUR * menuProgress),
            underStatusBar = tab == Tab.Serendipity,
            overlay = {
                opened?.let { open ->
                    BackHandler { opened = null }
                    PageTheme(scheme) {
                        CardSheet(
                            open.card,
                            onClose = { opened = null },
                            onMenu = { menuOpen = true },
                            onRemoveTag = { tag ->
                                scope.launch {
                                    when (val answer = viewModel.removeTag(open.card, tag)) {
                                        is ApiResult.Ok -> updateCard(answer.value)
                                        is ApiResult.Failed -> failure = answer.failure
                                    }
                                }
                            },
                        )
                    }
                }
                if (menuProgress > 0f) Box(Modifier.fillMaxSize().background(DarkMenuColors.scrim.copy(alpha = DarkMenuColors.scrim.alpha * menuProgress)))
            },
        ) {
            when (tab) {
                Tab.Everything -> EverythingScreen(
                    viewModel,
                    onOpenCard = { openCard(it, null) },
                    onOpenSettings = { scope.launch { drawerState.open() } },
                    onAdd = {},
                )
                Tab.Spaces -> SpacesScreen(
                    viewModel,
                    openSpace,
                    onOpenSpace = { openSpace = it },
                    onOpenCard = openCard,
                    onNewSpace = { newSpaceOpen = true },
                    onDeleteSpace = { deleting = it },
                )
                Tab.Serendipity -> SerendipityScreen(viewModel, onOpenCard = { openCard(it, null) })
            }
        }
    }

    val open = opened
    if (menuOpen && open != null) {
        CardMenuSheet(
            items = cardMenuItems(open.card, open.spaceId, features),
            onSelect = { item ->
                scope.launch {
                    val opensSubmenu = !item.locked && (item.action == CardMenuAction.AddTags || item.action == CardMenuAction.AddToSpace)
                    if (!opensSubmenu) {
                        sheetState.hide()
                        menuOpen = false
                    }
                    when (val outcome = viewModel.perform(item, open.card, open.spaceId)) {
                        is CardMenuOutcome.OpenSubmenu -> submenu = outcome.submenu
                        is CardMenuOutcome.Copy -> clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("card", outcome.text)))
                        is CardMenuOutcome.OpenUrl -> runCatching { uriHandler.openUri(outcome.url) }
                        is CardMenuOutcome.Deleted, is CardMenuOutcome.RemovedFromSpace -> opened = null
                        is CardMenuOutcome.PinChanged -> opened = open.copy(card = outcome.card)
                        is CardMenuOutcome.Downloaded -> Toast.makeText(context, DOWNLOADED, Toast.LENGTH_SHORT).show()
                        is CardMenuOutcome.ShowSimilar -> {
                            opened = null
                            tab = Tab.Everything
                        }
                        is CardMenuOutcome.Failed -> failure = outcome.failure
                        is CardMenuOutcome.NotAvailable -> Unit
                    }
                }
            },
            onDismiss = {
                menuOpen = false
                submenu = null
            },
            sheetState = sheetState,
            submenu = submenu?.let { shown ->
                @Composable {
                    when (shown) {
                        CardSubmenu.Tags -> AddTagsPage(
                            allTags = tagChoices.all,
                            recentTags = tagChoices.recent,
                            onClose = closeMenu,
                            onSave = { names ->
                                closeMenu()
                                scope.launch {
                                    val card = opened?.card ?: return@launch
                                    when (val answer = viewModel.addTags(card, names)) {
                                        is ApiResult.Ok -> updateCard(answer.value)
                                        is ApiResult.Failed -> failure = answer.failure
                                    }
                                }
                            },
                        )
                        CardSubmenu.Spaces -> SelectSpacePage(
                            spaces = spaces.spaces,
                            selected = open.card.spaces.orEmpty().toSet(),
                            onClose = closeMenu,
                            onToggle = { space ->
                                scope.launch {
                                    val card = opened?.card ?: return@launch
                                    when (val answer = viewModel.toggleSpace(card, space)) {
                                        is ApiResult.Ok -> updateCard(answer.value)
                                        is ApiResult.Failed -> failure = answer.failure
                                    }
                                }
                            },
                            onCreateSpace = {
                                closeMenu()
                                newSpaceOpen = true
                            },
                        )
                    }
                }
            },
        )
    }
    if (newSpaceOpen) {
        PageTheme(scheme) {
            NewSpaceDialog(
                onDismiss = { newSpaceOpen = false },
                onCreate = { name, color ->
                    newSpaceOpen = false
                    scope.launch { (viewModel.createSpace(name, color) as? ApiResult.Failed)?.let { failure = it.failure } }
                },
            )
        }
    }
    deleting?.let { space ->
        AlertDialog(
            onDismissRequest = { deleting = null },
            title = { Text(DELETE_SPACE_TITLE) },
            text = { Text(DELETE_SPACE_TEXT) },
            confirmButton = {
                TextButton(onClick = {
                    deleting = null
                    scope.launch {
                        when (val answer = viewModel.deleteSpace(space)) {
                            is ApiResult.Ok -> if (openSpace?.id == space.id) openSpace = null
                            is ApiResult.Failed -> failure = answer.failure
                        }
                    }
                }) { Text("Delete space") }
            },
            dismissButton = { TextButton(onClick = { deleting = null }) { Text("Cancel") } },
        )
    }
    failure?.let { ApiErrorDialog(title = null, message = describe(it), onDismiss = { failure = null }) }
}

private const val DOWNLOADED = "Successfully downloaded"
private const val DELETE_SPACE_TITLE = "Delete this space?"
private const val DELETE_SPACE_TEXT = "The cards in it stay in your mind."

/**
 * The frame of the signed-in screens: the page between the status bar and the tab bar in the account's appearance,
 * with the status bar background and the tab bar in the system's.
 *
 * @param tab Tab shown as selected.
 * @param onSelectTab Called with the tab the user tapped.
 * @param scheme Appearance the account picked for the pages.
 * @param modifier Modifier for the full-screen box.
 * @param underStatusBar True lets the page reach under the status bar; the page then pads its own content.
 * @param overlay Drawn over the page and the tab bar, such as an opened card.
 * @param page The selected tab's page.
 */
@Composable
fun ShellFrame(
    tab: Tab,
    onSelectTab: (Tab) -> Unit,
    scheme: PageScheme,
    modifier: Modifier = Modifier,
    underStatusBar: Boolean = false,
    overlay: @Composable BoxScope.() -> Unit = {},
    page: @Composable () -> Unit,
) {
    Box(modifier.fillMaxSize().background(MymindTheme.colors.page)) {
        PageTheme(scheme) {
            val top = if (underStatusBar) Modifier else Modifier.statusBarsPadding()
            Box(Modifier.fillMaxSize().then(top).padding(bottom = PAGE_BOTTOM_GAP).background(MymindTheme.page.background)) {
                page()
            }
        }
        MymindTabBar(tab, onSelect = onSelectTab, modifier = Modifier.align(Alignment.BottomCenter))
        overlay()
    }
}
