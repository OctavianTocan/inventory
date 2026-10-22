package com.octaviantocan.mymind.ui.shell

import android.content.ClipData
import android.content.Intent
import android.provider.MediaStore
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.uploadFileOf
import com.octaviantocan.mymind.ui.card.CardDetailDialog
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.errors.ApiErrorDialog
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.library.EverythingScreen
import com.octaviantocan.mymind.ui.library.GalleryControls
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.library.NewSpaceDialog
import com.octaviantocan.mymind.ui.library.SerendipityScreen
import com.octaviantocan.mymind.ui.library.SpacesScreen
import com.octaviantocan.mymind.ui.library.rememberPublicAccountPhotoLoader
import com.octaviantocan.mymind.ui.menu.CreateSpacePage
import com.octaviantocan.mymind.ui.menu.AddTagsPage
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuOutcome
import com.octaviantocan.mymind.ui.menu.CardMenuSheet
import com.octaviantocan.mymind.ui.menu.rememberCardMenuPosition
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.menu.SelectSpacePage
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.ui.settings.ProfileSheetHost
import com.octaviantocan.mymind.ui.settings.SettingsUpdates
import com.octaviantocan.mymind.ui.settings.SettingsViewModel
import com.octaviantocan.mymind.ui.theme.MymindTheme
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import com.octaviantocan.mymind.ui.upload.AddSheet
import com.octaviantocan.mymind.ui.upload.PICKABLE_FILES
import com.octaviantocan.mymind.ui.upload.UploadQueueStrip
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

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
 * Where the + button was tapped.
 *
 * @property spaceId Space the new cards join, or null outside a space.
 * @property title Title of the Add sheet.
 * @property newSpace True offers New space too, as the Spaces tab does.
 */
private data class AddTarget(val spaceId: String?, val title: String = "Add to your mind", val newSpace: Boolean = false)

/**
 * The signed-in shell: the page of the selected tab with the tab bar over its bottom edge, an opened card over both,
 * the card menu, the Add sheet with the upload queue strip, and the profile sheet.
 *
 * @param viewModel Library behind the tabs and the card menu.
 * @param settingsViewModel State and actions behind the profile sheet.
 * @param updates Update rows of the profile sheet.
 * @param extras Drawn over everything, such as the update sheets.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainShell(
    viewModel: LibraryViewModel,
    settingsViewModel: SettingsViewModel,
    updates: SettingsUpdates = SettingsUpdates(),
    extras: @Composable () -> Unit = {},
) {
    var tab by rememberSaveable { mutableStateOf(Tab.Everything) }
    var searchActive by rememberSaveable { mutableStateOf(false) }
    val pageStates = rememberSaveableStateHolder()
    var openSpace by remember { mutableStateOf<Space?>(null) }
    var opened by remember { mutableStateOf<OpenedCard?>(null) }
    var menuFor by remember { mutableStateOf<OpenedCard?>(null) }
    var menuOpen by remember { mutableStateOf(false) }
    var adding by remember { mutableStateOf<AddTarget?>(null) }
    var profileOpen by remember { mutableStateOf(false) }
    var uploadTarget by remember { mutableStateOf(AddTarget(null)) }
    var photo by remember { mutableStateOf<File?>(null) }
    var submenu by remember { mutableStateOf<CardSubmenu?>(null) }
    var newSpaceOpen by remember { mutableStateOf(false) }
    var deleting by remember { mutableStateOf<Space?>(null) }
    var failure by remember { mutableStateOf<ApiFailure?>(null) }
    val scheme by viewModel.scheme.collectAsStateWithLifecycle()
    val features by viewModel.features.collectAsStateWithLifecycle()
    val tagChoices by viewModel.tagChoices.collectAsStateWithLifecycle()
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    val cardSpaceCreation by viewModel.cardSpaceCreation.collectAsStateWithLifecycle()
    val uploads by viewModel.uploads.state.collectAsStateWithLifecycle()
    val query by viewModel.search.query.collectAsStateWithLifecycle()
    val filter by viewModel.search.filter.collectAsStateWithLifecycle()
    val account by settingsViewModel.state.collectAsStateWithLifecycle()
    val focusManager = LocalFocusManager.current
    val searching = tab == Tab.Everything && (searchActive || query.isNotEmpty() || filter != null)
    val closeSearch = { viewModel.search.clear(); searchActive = false; focusManager.clearFocus() }
    val selectTab: (Tab) -> Unit = { next -> focusManager.clearFocus(); searchActive = false; tab = next }
    LaunchedEffect(settingsViewModel) { settingsViewModel.refresh() }
    val photoLoader = if (account.photoUrl != null) rememberPublicAccountPhotoLoader() else null
    BackHandler(enabled = opened == null && !menuOpen && searching, onBack = closeSearch)
    BackHandler(enabled = opened == null && !menuOpen && tab == Tab.Serendipity) { tab = Tab.Everything }
    val sheetState = key(menuOpen, menuFor?.card?.id) { rememberCardMenuPosition() }
    val motion = rememberSheetMotionEnabled()
    val scope = rememberCoroutineScope()
    val clipboard = LocalClipboard.current
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    val menuProgress = if (menuOpen) sheetState.scrim else 0f
    val openCard = { card: Card, spaceId: String? -> opened = OpenedCard(card, spaceId) }
    val updateCard = { card: Card ->
        opened = opened?.takeIf { it.card.id == card.id }?.copy(card = card) ?: opened
        menuFor = menuFor?.takeIf { it.card.id == card.id }?.copy(card = card) ?: menuFor
    }
    val openMenu = { card: Card, spaceId: String? ->
        menuFor = OpenedCard(card, spaceId)
        menuOpen = true
    }
    val canTakePhoto = remember { Intent(MediaStore.ACTION_IMAGE_CAPTURE).resolveActivity(context.packageManager) != null }
    val pickMedia = rememberLauncherForActivityResult(ActivityResultContracts.PickMultipleVisualMedia()) { uris ->
        val spaceId = uploadTarget.spaceId
        scope.launch {
            val files = withContext(Dispatchers.IO) { uris.mapNotNull { uploadFileOf(context.contentResolver, it) } }
            viewModel.upload(files, spaceId)
        }
    }
    val pickFiles = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        val spaceId = uploadTarget.spaceId
        scope.launch {
            val files = withContext(Dispatchers.IO) { uris.mapNotNull { uploadFileOf(context.contentResolver, it) } }
            viewModel.upload(files, spaceId)
        }
    }
    val takePhoto = rememberLauncherForActivityResult(ActivityResultContracts.TakePicture()) { saved ->
        val file = photo
        if (saved && file != null) viewModel.upload(listOf(uploadFileOf(file)), uploadTarget.spaceId)
    }
    val closeMenu: () -> Unit = {
        sheetState.dismiss(scope, motion) {
            menuOpen = false
            submenu = null
        }
    }

    ShellFrame(
        tab = tab,
        onSelectTab = selectTab,
        scheme = scheme,
        modifier = Modifier.blur(MENU_BLUR * menuProgress),
        underStatusBar = tab == Tab.Serendipity,
        footer = {
            LibraryFootBar(tab, selectTab, searching, query, viewModel.search::setQuery,
                onSearch = { tab = Tab.Everything; searchActive = true }, onCloseSearch = closeSearch)
        },
        overlay = {
            opened?.let { open ->
                key(open.card.id) {
                    PageTheme(scheme) {
                        CardDetailDialog(
                            open.card,
                            onClose = { opened = null },
                            onMenu = { openMenu(open.card, open.spaceId) },
                            actionsProgress = menuProgress,
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
            }
            UploadQueueStrip(
                uploads,
                onToggle = viewModel.uploads::toggleDetails,
                onDismiss = viewModel.uploads::dismiss,
                modifier = Modifier.align(Alignment.BottomCenter),
                hidden = tab == Tab.Serendipity,
            )
        },
    ) {
        pageStates.SaveableStateProvider(tab.name) {
            when (tab) {
                Tab.Everything -> EverythingScreen(
                    viewModel,
                    onOpenCard = { openCard(it, null) },
                    onOpenSettings = { profileOpen = true },
                    onAdd = { adding = AddTarget(null) },
                    onLongPressCard = { openMenu(it, null) },
                    searching = searching,
                    onSerendipity = { focusManager.clearFocus(); tab = Tab.Serendipity },
                    photoUrl = account.photoUrl,
                )
                Tab.Spaces -> SpacesScreen(
                    viewModel,
                    openSpace,
                    onOpenSpace = { openSpace = it },
                    onOpenCard = openCard,
                    onDeleteSpace = { deleting = it },
                    onAddToSpace = { adding = AddTarget(it.id, "Add to ${it.name}") },
                    onLongPressCard = openMenu,
                ) {
                    GalleryControls(
                        onSettings = { profileOpen = true },
                        onAdd = { adding = AddTarget(null, newSpace = true) },
                        onSerendipity = { focusManager.clearFocus(); tab = Tab.Serendipity },
                        modifier = Modifier.align(Alignment.TopEnd),
                        photoUrl = account.photoUrl,
                        photoLoader = photoLoader,
                    )
                }
                Tab.Serendipity -> SerendipityScreen(viewModel, onOpenCard = { openCard(it, null) })
            }
        }
    }

    val target = menuFor
    if (menuOpen && target != null) {
        CardMenuSheet(
            items = cardMenuItems(target.card, target.spaceId, features),
            accent = cardAccent(target.card),
            onSelect = { item ->
                scope.launch {
                    val opensSubmenu = !item.locked && (item.action == CardMenuAction.AddTags || item.action == CardMenuAction.AddToSpace)
                    if (sheetState.closing) return@launch
                    if (!opensSubmenu) {
                        if (!sheetState.startClosing()) return@launch
                        sheetState.hide(motion)
                        menuOpen = false
                        submenu = null
                    }
                    when (val outcome = viewModel.perform(item, target.card, target.spaceId)) {
                        is CardMenuOutcome.OpenSubmenu -> submenu = outcome.submenu
                        is CardMenuOutcome.Copy -> clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("card", outcome.text)))
                        is CardMenuOutcome.OpenUrl -> runCatching { uriHandler.openUri(outcome.url) }
                        is CardMenuOutcome.Deleted, is CardMenuOutcome.RemovedFromSpace -> if (opened?.card?.id == target.card.id) opened = null
                        is CardMenuOutcome.PinChanged -> updateCard(outcome.card)
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
            position = sheetState,
            submenu = submenu,
            onBack = { focusManager.clearFocus(); submenu = submenu?.parent },
            submenuContent = { shown ->
                    when (shown) {
                        CardSubmenu.Tags -> AddTagsPage(
                            allTags = tagChoices.all,
                            recentTags = tagChoices.recent,
                            onSave = { names ->
                                closeMenu()
                                scope.launch {
                                    val card = menuFor?.card ?: return@launch
                                    when (val answer = viewModel.addTags(card, names)) {
                                        is ApiResult.Ok -> updateCard(answer.value)
                                        is ApiResult.Failed -> failure = answer.failure
                                    }
                                }
                            },
                        )
                        CardSubmenu.CreateSpace -> CreateSpacePage(
                            creation = cardSpaceCreation,
                            onCreate = viewModel::createCardSpace,
                            onCreated = {
                                if (submenu == CardSubmenu.CreateSpace) {
                                    viewModel.acknowledgeCardSpaceCreation()
                                    focusManager.clearFocus()
                                    submenu = CardSubmenu.Spaces
                                }
                            },
                        )
                        CardSubmenu.Spaces -> SelectSpacePage(
                            spaces = spaces.spaces,
                            selected = target.card.spaces.orEmpty().toSet(),
                            onToggle = { space ->
                                scope.launch {
                                    val card = menuFor?.card ?: return@launch
                                    when (val answer = viewModel.toggleSpace(card, space)) {
                                        is ApiResult.Ok -> updateCard(answer.value)
                                        is ApiResult.Failed -> failure = answer.failure
                                    }
                                }
                            },
                            onCreateSpace = {
                                viewModel.acknowledgeCardSpaceCreation()
                                submenu = CardSubmenu.CreateSpace
                            },
                        )
                    }
            },
        )
    }
    adding?.let { target ->
        AddSheet(
            title = target.title,
            onPhotos = {
                adding = null
                uploadTarget = target
                pickMedia.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageAndVideo))
            },
            onFiles = {
                adding = null
                uploadTarget = target
                pickFiles.launch(PICKABLE_FILES)
            },
            onCamera = if (canTakePhoto) {
                {
                    adding = null
                    uploadTarget = target
                    val file = File.createTempFile("take_picture", ".jpg", context.cacheDir)
                    photo = file
                    takePhoto.launch(FileProvider.getUriForFile(context, "${context.packageName}.provider", file))
                }
            } else {
                null
            },
            onClose = { adding = null },
            onNewSpace = if (target.newSpace) ({ adding = null; newSpaceOpen = true }) else null,
        )
    }
    if (profileOpen) ProfileSheetHost(settingsViewModel, updates, onClose = { profileOpen = false })
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
    extras()
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
    footer: (@Composable () -> Unit)? = null,
    overlay: @Composable BoxScope.() -> Unit = {},
    page: @Composable () -> Unit,
) {
    val bars = remember { FootBars() }
    CompositionLocalProvider(LocalFootBars provides bars) {
        Box(modifier.fillMaxSize().background(MymindTheme.colors.page)) {
            PageTheme(scheme) {
                val top = if (underStatusBar) Modifier else Modifier.statusBarsPadding()
                val library = tab == Tab.Everything || tab == Tab.Spaces
                val background = if (library) LibraryStyle.colors.background else MymindTheme.page.background
                Box(Modifier.fillMaxSize().then(top).background(background).padding(bottom = if (library) 0.dp else PAGE_BOTTOM_GAP)) {
                    page()
                }
                if (footer != null) footer() else MymindTabBar(tab, onSelectTab)
                FootBarLayer(bars, Modifier.align(Alignment.BottomCenter))
            }
            overlay()
        }
    }
}
