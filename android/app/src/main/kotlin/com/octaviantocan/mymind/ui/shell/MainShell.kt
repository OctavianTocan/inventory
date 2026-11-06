package com.octaviantocan.mymind.ui.shell

import com.octaviantocan.mymind.ui.theme.themeBackground
import android.content.ClipData
import android.content.Intent
import android.provider.MediaStore
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.Text
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.Stable
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInWindow
import androidx.compose.ui.text.style.TextAlign
import com.octaviantocan.mymind.ui.theme.LocalPaneSpan
import com.octaviantocan.mymind.ui.theme.PaneSpan
import com.octaviantocan.mymind.ui.theme.ReadableWidth
import com.octaviantocan.mymind.ui.theme.SideInsets
import com.octaviantocan.mymind.ui.theme.WindowShape
import com.octaviantocan.mymind.ui.theme.listPaneWidth
import com.octaviantocan.mymind.ui.theme.readableWidth
import com.octaviantocan.mymind.ui.theme.rememberWindowShape
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.data.ApiFailure
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.data.uploadFileOf
import com.octaviantocan.mymind.ui.card.CardDetailDialog
import com.octaviantocan.mymind.ui.card.CardDetailPane
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.card.CardSheetMotion
import com.octaviantocan.mymind.ui.card.FetchedPdfs
import com.octaviantocan.mymind.ui.card.ReaderSheetLayout
import com.octaviantocan.mymind.ui.card.ReaderSheetState
import com.octaviantocan.mymind.ui.card.LocalPdfSource
import com.octaviantocan.mymind.ui.card.ReaderActions
import com.octaviantocan.mymind.ui.card.shrunk
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.upload.UPLOAD_STRIP_BOTTOM
import com.octaviantocan.mymind.ui.card.ReaderDetails
import com.octaviantocan.mymind.ui.describe
import com.octaviantocan.mymind.ui.errors.ApiErrorDialog
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.library.pinned
import com.octaviantocan.mymind.ui.library.EverythingScreen
import com.octaviantocan.mymind.ui.library.CardChoice
import com.octaviantocan.mymind.ui.library.ChoiceCopySheet
import com.octaviantocan.mymind.ui.library.ChoicePackBar
import com.octaviantocan.mymind.ui.library.PackOut
import com.octaviantocan.mymind.ui.library.picturesLabel
import com.octaviantocan.mymind.ui.library.ChoiceSpaceSheet
import com.octaviantocan.mymind.ui.library.ChoiceTagSheet
import com.octaviantocan.mymind.ui.library.ChoiceUndoBar
import com.octaviantocan.mymind.ui.library.ManyChange
import com.octaviantocan.mymind.ui.library.UNDO_MANY_MS
import com.octaviantocan.mymind.ui.library.findsLabel
import kotlinx.coroutines.delay
import androidx.compose.foundation.layout.fillMaxWidth
import com.octaviantocan.mymind.ui.library.GalleryControls
import com.octaviantocan.mymind.ui.library.LibraryViewModel
import com.octaviantocan.mymind.ui.menu.NewSpaceSheet
import com.octaviantocan.mymind.ui.library.SpacesScreen
import com.octaviantocan.mymind.ui.library.rememberPublicAccountPhotoLoader
import androidx.compose.ui.graphics.asAndroidBitmap
import com.octaviantocan.mymind.ui.card.PartAction
import com.octaviantocan.mymind.ui.card.partText
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.menu.CardImagePage
import com.octaviantocan.mymind.ui.menu.NotePage
import com.octaviantocan.mymind.ui.menu.PartContent
import com.octaviantocan.mymind.ui.menu.agentLink
import com.octaviantocan.mymind.ui.menu.cacheImage
import com.octaviantocan.mymind.ui.menu.DownloadsSaver
import com.octaviantocan.mymind.ui.menu.manyWords
import com.octaviantocan.mymind.ui.menu.savePicture
import com.octaviantocan.mymind.ui.menu.loadPicture
import com.octaviantocan.mymind.ui.menu.sharePicture
import com.octaviantocan.mymind.ui.menu.sharedImageName
import com.octaviantocan.mymind.ui.menu.AllSpacesPage
import com.octaviantocan.mymind.ui.menu.InlineSpaces
import com.octaviantocan.mymind.ui.menu.SpaceChoices
import com.octaviantocan.mymind.ui.menu.UndoDeleteBar
import com.octaviantocan.mymind.ui.menu.suggestSpace
import com.octaviantocan.mymind.ui.menu.AddTagsPage
import com.octaviantocan.mymind.ui.menu.CardMenuAction
import com.octaviantocan.mymind.ui.menu.CardMenuItem
import com.octaviantocan.mymind.ui.menu.CardMenuOutcome
import com.octaviantocan.mymind.ui.menu.CardSheetRoot
import com.octaviantocan.mymind.ui.menu.ShareContent
import com.octaviantocan.mymind.ui.menu.shareText
import com.octaviantocan.mymind.ui.menu.CardMenuSheet
import com.octaviantocan.mymind.ui.menu.rememberCardMenuPosition
import com.octaviantocan.mymind.ui.card.rememberSheetMotionEnabled
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.feedback.FeedbackTracker
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

/** Index of Ask among the library's side pages, after Everything and Spaces. */
private const val ASK_PAGE = 2

/** Everything, Spaces and Ask. */
private const val PAGE_COUNT = 3

/** Distance from the bottom of the screen to the end of the library page; the tab bar covers the rest. */
val PAGE_BOTTOM_GAP = 80.dp

private val MENU_BLUR = 8.dp

/**
 * A card the user opened.
 *
 * @property card The card.
 * @property spaceId Space it was opened from, or null.
 * @property reading True while it shows in Reader Mode.
 * @property editingTags True while its tags show the crosses that take them off.
 */
data class OpenedCard(val card: Card, val spaceId: String?, val reading: Boolean = false, val editingTags: Boolean = false)

/**
 * Where the + button was tapped.
 *
 * @property spaceId Space the new cards join, or null outside a space.
 * @property title Title of the Add sheet.
 */
private data class AddTarget(val spaceId: String?, val title: String = "Add to your inventory")

/**
 * The signed-in shell: the page of the selected tab with the tab bar over its bottom edge, an opened card over both,
 * the card menu, the Add sheet with the upload queue strip, and the profile sheet.
 *
 * @param viewModel Library behind the tabs and the card menu.
 * @param settingsViewModel State and actions behind the profile sheet.
 * @param updates Update rows of the profile sheet.
 * @param ai The account's Claude Code and Codex, for the profile sheet's AI rows, or null for none.
 * @param feedback Sends feedback from the profile sheet, or null for a sheet without the Send feedback row.
 * @param extras Drawn over everything, such as the update sheets.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainShell(
    viewModel: LibraryViewModel,
    settingsViewModel: SettingsViewModel,
    updates: SettingsUpdates = SettingsUpdates(),
    ai: AiAccount? = null,
    feedback: FeedbackTracker? = null,
    extras: @Composable () -> Unit = {},
) {
    var tab by rememberSaveable { mutableStateOf(Tab.Everything) }
    var searchActive by rememberSaveable { mutableStateOf(false) }
    // Opening Ask, by its button or a swipe, puts the cursor in its field; showing similar cards leaves the keyboard down.
    var askFocus by remember { mutableStateOf(true) }
    var openSpace by remember { mutableStateOf<Space?>(null) }
    var opened by remember { mutableStateOf<OpenedCard?>(null) }
    var menuFor by remember { mutableStateOf<OpenedCard?>(null) }
    var menuOpen by remember { mutableStateOf(false) }
    var adding by remember { mutableStateOf<AddTarget?>(null) }
    var profileOpen by remember { mutableStateOf(false) }
    var uploadTarget by remember { mutableStateOf(AddTarget(null)) }
    var photo by remember { mutableStateOf<File?>(null) }
    var submenu by remember { mutableStateOf<CardSubmenu?>(null) }
    var sheetRoot by remember { mutableStateOf<CardSheetRoot>(CardSheetRoot.Actions) }
    // Cards opened before the one on screen, latest first, for the reader's name card.
    val recent = remember { mutableStateListOf<Card>() }
    var newSpaceOpen by remember { mutableStateOf(false) }
    var creatingSpace by remember { mutableStateOf(false) }
    var newSpaceError by remember { mutableStateOf<String?>(null) }
    val openNewSpace = { newSpaceError = null; newSpaceOpen = true }
    var failure by remember { mutableStateOf<ApiFailure?>(null) }
    // The finds gathered by holding and sliding, kept across pages until emptied.
    val choice = remember { CardChoice() }
    var packSheet by remember { mutableStateOf<PackSheet?>(null) }
    var manyDone by remember { mutableStateOf<Pair<String, ManyChange>?>(null) }
    var deleteHold by remember { mutableFloatStateOf(0f) }
    // The reader a delete closed, to open again on Undo.
    var undoReopens by remember { mutableStateOf<OpenedCard?>(null) }
    val pendingDelete by viewModel.deleting.collectAsStateWithLifecycle()
    val scheme by viewModel.scheme.collectAsStateWithLifecycle()
    val features by viewModel.features.collectAsStateWithLifecycle()
    val tagChoices by viewModel.tagChoices.collectAsStateWithLifecycle()
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    val cardSpaceCreation by viewModel.cardSpaceCreation.collectAsStateWithLifecycle()
    val uploads by viewModel.uploads.state.collectAsStateWithLifecycle()
    val query by viewModel.search.query.collectAsStateWithLifecycle()
    val searchFilter by viewModel.search.filter.collectAsStateWithLifecycle()
    val account by settingsViewModel.state.collectAsStateWithLifecycle()
    val focusManager = LocalFocusManager.current
    val searching = searchActive
    val closeSearch = { viewModel.search.clear(); searchActive = false; focusManager.clearFocus() }
    val selectTab: (Tab) -> Unit = { next -> focusManager.clearFocus(); searchActive = false; tab = next }
    // Everything, Spaces and Ask sit side by side in the bar's order, the circle's Ask last (Tavi, 5 November 2026).
    val page = if (searching) ASK_PAGE else tab.ordinal
    val pages = rememberSidePages(page, PAGE_COUNT) { next ->
        if (next == ASK_PAGE) {
            askFocus = true
            searchActive = true
        } else {
            selectTab(Tab.entries[next])
        }
    }
    LaunchedEffect(settingsViewModel) { settingsViewModel.refresh() }
    LaunchedEffect(page, openSpace?.id) { Track.event("screen", screen = if (openSpace != null) "space" else screenName(page)) }
    val photoLoader = if (account.photoUrl != null) rememberPublicAccountPhotoLoader() else null
    BackHandler(enabled = opened == null && !menuOpen && searching, onBack = closeSearch)
    // Registered after the search's, so Back empties the backpack before it leaves the search.
    BackHandler(enabled = opened == null && !menuOpen && choice.active) { choice.clear() }
    LaunchedEffect(manyDone) { if (manyDone != null) { delay(UNDO_MANY_MS); manyDone = null } }
    val sheetState = key(menuOpen, menuFor?.card?.id) { rememberCardMenuPosition() }
    val motion = rememberSheetMotionEnabled()
    val scope = rememberCoroutineScope()
    val clipboard = LocalClipboard.current
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    val menuProgress = if (menuOpen) sheetState.scrim else 0f
    val shrink by animateFloatAsState(if (motion) deleteHold else 0f, if (deleteHold == 0f) tween(CardSheetMotion.FAST) else snap(), label = "deleteShrink")
    LaunchedEffect(viewModel) { viewModel.deleteFailures.collect { failure = it } }
    LaunchedEffect(pendingDelete) { if (pendingDelete == null) undoReopens = null }
    val keepRecent = { card: Card -> recent.removeAll { it.id == card.id }; recent.add(0, card); if (recent.size > RECENT_KEPT) recent.removeAt(recent.lastIndex) }
    // The reader rises as a sheet over the gallery, which steps back behind it; a card opened from the reader takes the
    // sheet's place without it moving.
    val readerSheet = remember { ReaderSheetState() }
    val openCard = { card: Card, spaceId: String? ->
        opened?.card?.takeIf { it.id != card.id }?.let(keepRecent)
        val rising = opened == null || readerSheet.closing
        opened = OpenedCard(card, spaceId)
        Track.event("open", screen = if (spaceId != null) "space" else screenName(page), card = card.id, kind = card.type)
        if (rising) readerSheet.show(scope, motion)
    }
    val reopen = { card: OpenedCard -> opened = card; readerSheet.show(scope, motion) }
    val gone = { keep: Boolean -> if (keep) opened?.card?.let(keepRecent); opened = null; readerSheet.reset() }
    val closeCard = { readerSheet.hide(scope, motion) { gone(true) } }
    // Closes the reader without keeping it among the recent cards, as for a card that is going.
    val dropCard = { readerSheet.hide(scope, motion) { gone(false) } }
    val read = { card: Card, spaceId: String? ->
        if (opened?.card?.id == card.id) opened = opened?.copy(reading = true)
        else {
            openCard(card, spaceId)
            opened = opened?.copy(reading = true)
        }
    }
    // The opened card follows the phone's copy too, such as a shared link the server has just read.
    val openedId = opened?.card?.id
    // The reader keeps its place when the window changes shape, as a phone turns or a foldable opens.
    val readerScroll = remember(openedId) { ScrollState(0) }
    val unfolded = rememberWindowShape() == WindowShape.Unfolded
    val panes = remember { ShellPanes() }
    // Opened PDFs, fetched once into the phone's cache so the reader draws their pages.
    val pdfs = remember(viewModel, context) { FetchedPdfs(File(context.cacheDir, PDF_DIR)) { card, file -> viewModel.fetchFile(card, file) } }
    // The cards nearest an opened product, for its Same Vibe section, kept with the id they were found for.
    var sameVibe by remember { mutableStateOf<Pair<String, List<Card>>?>(null) }
    val sameVibeOpen = "sameVibe" in features
    LaunchedEffect(openedId, sameVibeOpen) {
        val card = opened?.card?.takeIf { it.type == "Product" && sameVibeOpen } ?: return@LaunchedEffect
        sameVibe = card.id to viewModel.sameVibe(card)
    }
    LaunchedEffect(openedId) {
        if (openedId == null) return@LaunchedEffect
        viewModel.redrawn.collect {
            val fresh = viewModel.cardNow(openedId) ?: return@collect
            opened = opened?.takeIf { it.card.id == openedId && it.card != fresh }?.copy(card = fresh) ?: opened
        }
    }
    val updateCard = { card: Card ->
        opened = opened?.takeIf { it.card.id == card.id }?.copy(card = card) ?: opened
        menuFor = menuFor?.takeIf { it.card.id == card.id }?.copy(card = card) ?: menuFor
    }
    val joinedNewSpace = cardSpaceCreation.card
    LaunchedEffect(joinedNewSpace) { joinedNewSpace?.let(updateCard) }
    val openSheet = { card: Card, spaceId: String?, root: CardSheetRoot, page: CardSubmenu? ->
        if (page == CardSubmenu.Tags) viewModel.loadTagChoices()
        menuFor = OpenedCard(card, spaceId)
        sheetRoot = root
        submenu = page
        menuOpen = true
    }
    val openMenu = { card: Card, spaceId: String? -> openSheet(card, spaceId, CardSheetRoot.Actions, null) }
    val send = { text: String, subject: String? ->
        val intent = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, text)
        subject?.let { intent.putExtra(Intent.EXTRA_SUBJECT, it) }
        Track.event("share", detail = "text")
        runCatching { context.startActivity(Intent.createChooser(intent, null)) }
        Unit
    }
    val copyLink = { card: Card -> card.url?.let { url -> scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("link", url))) } }; Unit }
    val copyAgentLink = { card: Card ->
        Track.event("share", card = card.id, kind = card.type, detail = "agent link")
        scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("Inventory card link", agentLink(card)))) }
        Unit
    }
    // Copies the card's picture or hands it to another app, through a file in the cache the app's provider lends out.
    val pictureOut = { card: Card, copy: Boolean ->
        scope.launch {
            val bitmap = loadPicture(context, card)
            if (bitmap == null) {
                Toast.makeText(context, NO_PICTURE, Toast.LENGTH_SHORT).show()
                return@launch
            }
            val uri = cacheImage(context, bitmap, sharedImageName(card, ".jpg"), png = false)
            if (copy) clipboard.setClipEntry(ClipEntry(ClipData.newUri(context.contentResolver, "picture", uri)))
            else sharePicture(context, uri, "image/jpeg")
        }
        Unit
    }
    val saver = remember(context) { DownloadsSaver(context.contentResolver) }
    // Saves a picture that has no file of its own, as the gallery draws it.
    val pictureSaved = { card: Card ->
        scope.launch {
            when (val saved = savePicture(context, card, saver)) {
                null -> Toast.makeText(context, NO_PICTURE, Toast.LENGTH_SHORT).show()
                is ApiResult.Ok -> Toast.makeText(context, DOWNLOADED, Toast.LENGTH_SHORT).show()
                is ApiResult.Failed -> failure = saved.failure
            }
        }
        Unit
    }
    // Copies or saves the backpack's finds together.
    val packOut = { pick: PackOut, cards: List<Card> ->
        scope.launch {
            Track.event("action", detail = "CopyMany:${pick.name}", results = cards.size)
            when (pick) {
                PackOut.Pictures -> {
                    val withPicture = cards.filter { galleryPicture(it) != null }
                    val uris = withPicture.mapNotNull { card -> loadPicture(context, card)?.let { cacheImage(context, it, "${card.id}.jpg", png = false) } }
                    if (uris.isEmpty()) {
                        Toast.makeText(context, NO_PICTURE, Toast.LENGTH_SHORT).show()
                        return@launch
                    }
                    val clip = ClipData.newUri(context.contentResolver, "pictures", uris.first())
                    uris.drop(1).forEach { clip.addItem(ClipData.Item(it)) }
                    clipboard.setClipEntry(ClipEntry(clip))
                    if (uris.size < withPicture.size) Toast.makeText(context, "Copied ${uris.size} of ${withPicture.size} pictures", Toast.LENGTH_SHORT).show()
                }
                PackOut.Words -> manyWords(cards)?.let { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("Inventory finds", it))) }
                PackOut.AgentLinks ->
                    clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("Inventory card links", cards.joinToString("\n") { agentLink(it) })))
                PackOut.SavePictures -> {
                    val withPicture = cards.filter { galleryPicture(it) != null }
                    var saved = 0
                    withPicture.forEach { card ->
                        val result = if (card.blob?.path != null && card.blob.type?.startsWith("image/") == true) {
                            if (viewModel.perform(CardMenuItem(CardMenuAction.Download), card, null) is CardMenuOutcome.Downloaded) 1 else 0
                        } else {
                            if (savePicture(context, card, saver, "${card.id}.jpg") is ApiResult.Ok) 1 else 0
                        }
                        saved += result
                    }
                    val text = if (saved == withPicture.size) "Saved ${picturesLabel(saved)} to Downloads"
                    else "Saved $saved of ${picturesLabel(withPicture.size)} to Downloads"
                    Toast.makeText(context, text, Toast.LENGTH_SHORT).show()
                }
            }
        }
        Unit
    }
    val togglePin = { card: Card ->
        scope.launch {
            val pin = CardMenuItem(if (card.pinned) CardMenuAction.Unpin else CardMenuAction.Pin)
            when (val outcome = viewModel.perform(pin, card, null)) {
                is CardMenuOutcome.PinChanged -> updateCard(outcome.card)
                is CardMenuOutcome.Failed -> failure = outcome.failure
                else -> Unit
            }
        }
        Unit
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

    // The opened card: a sheet over the gallery, or beside it on an unfolded window (Pully's rule 206).
    val reader: @Composable (Boolean) -> Unit = { inPane ->
        opened?.let { open ->
            key(open.card.id) {
                val actions = ReaderActions(
                    onShare = { openSheet(open.card, open.spaceId, CardSheetRoot.Share, null) },
                    onAddTags = { openSheet(open.card, open.spaceId, CardSheetRoot.Actions, CardSubmenu.Tags) },
                    onCopyLink = { copyLink(open.card) },
                    onTogglePin = { togglePin(open.card) },
                    onSpaces = { openSheet(open.card, open.spaceId, CardSheetRoot.Actions, CardSubmenu.Spaces) },
                    onOpenRecent = { openCard(it, null) },
                    onEverything = {
                        closeCard()
                        selectTab(Tab.Everything)
                    },
                    onPart = { part -> openSheet(open.card, open.spaceId, CardSheetRoot.Part(part), null) },
                    onReading = { on -> opened = opened?.takeIf { it.card.id == open.card.id }?.copy(reading = on) ?: opened },
                    onEditingTags = { on -> opened = opened?.takeIf { it.card.id == open.card.id }?.copy(editingTags = on) ?: opened },
                )
                val details = ReaderDetails(
                    spaces = spaces.spaces.filter { it.id in open.card.spaces.orEmpty() }.mapNotNull { it.name },
                    recent = recent.toList(),
                    sameVibe = sameVibe?.takeIf { it.first == open.card.id }?.second.orEmpty(),
                )
                val removeTag = { tag: Tag ->
                    scope.launch {
                        when (val answer = viewModel.removeTag(open.card, tag)) {
                            is ApiResult.Ok -> updateCard(answer.value)
                            is ApiResult.Failed -> failure = answer.failure
                        }
                    }
                    Unit
                }
                PageTheme(scheme) {
                    CompositionLocalProvider(LocalPdfSource provides pdfs) {
                        if (inPane) {
                            CardDetailPane(open.card, onClose = closeCard, onMenu = { openMenu(open.card, open.spaceId) }, onRemoveTag = removeTag,
                                actionsProgress = menuProgress, shrink = shrink, reading = open.reading, editingTags = open.editingTags,
                                scrollState = readerScroll, actions = actions, details = details, sheet = readerSheet)
                        } else {
                            CardDetailDialog(open.card, onClose = closeCard, onMenu = { openMenu(open.card, open.spaceId) }, onRemoveTag = removeTag,
                                actionsProgress = menuProgress, shrink = shrink, reading = open.reading, editingTags = open.editingTags,
                                scrollState = readerScroll, actions = actions, details = details, sheet = readerSheet,
                                onPulledAway = { gone(true) })
                        }
                    }
                }
            }
        }
    }

    ShellFrame(
        tab = tab,
        onSelectTab = selectTab,
        scheme = scheme,
        modifier = Modifier.blur(MENU_BLUR * menuProgress).shrunk { if (opened == null) shrink else 0f },
        recede = { if (opened != null && !unfolded) readerSheet.recede() else 0f },
        panes = panes,
        beside = if (unfolded) ({ if (opened == null) NothingOpen() else reader(true) }) else null,
        footer = {
            LibraryFootBar(tab, selectTab, searching, query, viewModel.search::setQuery,
                onSearch = { askFocus = true; searchActive = true }, onCloseSearch = closeSearch,
                onSubmit = viewModel.search::submit, focusOnOpen = askFocus,
                hint = searchFilter?.takeUnless { it.similar }?.let { "Search in ${it.label}" } ?: SEARCH_HINT)
        },
        overlay = {
            if (!unfolded) reader(false)
            UploadQueueStrip(
                uploads,
                onToggle = viewModel.uploads::toggleDetails,
                onDismiss = viewModel.uploads::dismiss,
                modifier = Modifier.align(Alignment.BottomCenter).widthIn(max = ReadableWidth),
            )
            if (choice.active && opened == null) {
                ChoicePackBar(choice.count,
                    onTag = { viewModel.loadTagChoices(); packSheet = PackSheet.Tags },
                    onSpace = { if (!spaces.loaded) viewModel.loadSpaces(); packSheet = PackSheet.Spaces },
                    onCopy = { packSheet = PackSheet.Copy },
                    onClose = { choice.clear() },
                    modifier = Modifier.align(Alignment.BottomCenter).padding(start = 16.dp, end = 16.dp, bottom = UPLOAD_STRIP_BOTTOM)
                        .widthIn(max = ReadableWidth - 32.dp).fillMaxWidth())
            }
            manyDone?.let { (text, change) ->
                key(change) {
                    ChoiceUndoBar(text,
                        onUndo = {
                            manyDone = null
                            scope.launch { choice.refresh(viewModel.undoMany(change)) }
                        },
                        modifier = Modifier.align(Alignment.BottomCenter)
                            .padding(start = 16.dp, end = 16.dp, bottom = UPLOAD_STRIP_BOTTOM + if (choice.active) PACK_BAR_HEIGHT else 0.dp)
                            .widthIn(max = ReadableWidth - 32.dp).fillMaxWidth())
                }
            }
            pendingDelete?.let { card ->
                key(card.id) {
                    UndoDeleteBar(
                        cardHeading(card),
                        onUndo = {
                            viewModel.undoDelete()
                            undoReopens?.let(reopen)
                        },
                        modifier = Modifier.align(Alignment.BottomCenter).padding(start = 16.dp, end = 16.dp, bottom = UPLOAD_STRIP_BOTTOM)
                            .widthIn(max = ReadableWidth - 32.dp),
                    )
                }
            }
        },
    ) {
        SidePages(pages, PAGE_COUNT) { shown ->
            when (shown) {
                ASK_PAGE -> EverythingScreen(
                    viewModel,
                    onOpenCard = { openCard(it, null) },
                    onOpenSettings = { profileOpen = true },
                    onAdd = { adding = AddTarget(null) },
                    onLongPressCard = { openMenu(it, null) },
                    choice = choice,
                    searching = true,
                    photoUrl = account.photoUrl,
                    openedLately = recent.toList(),
                    onOpenSpace = { found ->
                        openSpace = viewModel.spaces.value.spaces.firstOrNull { it.id == found.id } ?: Space(found.id, found.name)
                        selectTab(Tab.Spaces)
                    },
                    photoLoader = photoLoader,
                )
                Tab.Everything.ordinal -> EverythingScreen(
                    viewModel,
                    onOpenCard = { openCard(it, null) },
                    cardOpen = opened != null,
                    onOpenSettings = { profileOpen = true },
                    onAdd = { adding = AddTarget(null) },
                    onLongPressCard = { openMenu(it, null) },
                    choice = choice,
                    photoUrl = account.photoUrl,
                    photoLoader = photoLoader,
                )
                else -> SpacesScreen(
                    viewModel,
                    openSpace,
                    onOpenSpace = { openSpace = it },
                    onOpenCard = openCard,
                    onDeleteSpace = { space ->
                        scope.launch {
                            when (val answer = viewModel.deleteSpace(space)) {
                                is ApiResult.Ok -> if (openSpace?.id == space.id) openSpace = null
                                is ApiResult.Failed -> failure = answer.failure
                            }
                        }
                    },
                    onAddToSpace = { adding = AddTarget(it.id, "Add to ${it.name}") },
                    onRenameSpace = { space, name ->
                        scope.launch {
                            when (val answer = viewModel.renameSpace(space, name)) {
                                is ApiResult.Ok -> if (openSpace?.id == space.id) openSpace = openSpace?.copy(name = name)
                                is ApiResult.Failed -> failure = answer.failure
                            }
                        }
                    },
                    onLongPressCard = openMenu,
                    onNewSpace = openNewSpace,
                ) {
                    GalleryControls(
                        onSettings = { profileOpen = true },
                        // On Spaces the + can only mean a new space, so it opens that sheet straight away.
                        onAdd = openNewSpace,
                        modifier = Modifier.align(Alignment.TopEnd),
                        photoUrl = account.photoUrl,
                        photoLoader = photoLoader,
                    )
                }
            }
        }
    }

    val target = menuFor
    if (menuOpen && target != null) {
        LaunchedEffect(Unit) { if (!spaces.loaded) viewModel.loadSpaces() }
        val suggestion = remember(target.card.id, spaces.spaces) { suggestSpace(target.card, viewModel.everything.state.value.cards, spaces.spaces) }
        val choices = SpaceChoices(spaces.spaces, target.card.spaces.orEmpty().toSet(), spaces.latest, suggestion)
        val toggleSpace = { space: Space ->
            scope.launch {
                val card = menuFor?.card ?: return@launch
                when (val answer = viewModel.toggleSpace(card, space)) {
                    is ApiResult.Ok -> updateCard(answer.value)
                    is ApiResult.Failed -> failure = answer.failure
                }
            }
            Unit
        }
        // Unfolded, the card's sheet rises over the pane it was opened from: the reader's card over the reader.
        val menuSpan = if (!unfolded) null else if (opened?.card?.id == target.card.id) panes.beside.value else panes.list.value
        CompositionLocalProvider(LocalPaneSpan provides menuSpan) {
        CardMenuSheet(
            items = cardMenuItems(target.card, target.spaceId, features),
            accent = cardAccent(target.card),
            onSelect = { item ->
                scope.launch {
                    val opensSubmenu = !item.locked && (item.action == CardMenuAction.AddTags || item.action == CardMenuAction.AddToSpace)
                    // Rotate keeps the sheet open, so each tap turns the picture another quarter.
                    val staysOpen = opensSubmenu || item.action == CardMenuAction.Rotate
                    if (sheetState.closing) return@launch
                    if (!staysOpen) {
                        if (!sheetState.startClosing()) return@launch
                        sheetState.hide(motion)
                        menuOpen = false
                        submenu = null
                    }
                    if (item.action == CardMenuAction.DeleteCard && !item.locked) {
                        undoReopens = opened?.takeIf { it.card.id == target.card.id }
                        if (undoReopens != null) dropCard()
                        recent.removeAll { it.id == target.card.id }
                        viewModel.deleteLater(target.card)
                        return@launch
                    }
                    when (val outcome = viewModel.perform(item, target.card, target.spaceId)) {
                        is CardMenuOutcome.OpenSubmenu -> submenu = outcome.submenu
                        is CardMenuOutcome.Copy -> clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("card", outcome.text)))
                        is CardMenuOutcome.CopyPicture -> pictureOut(outcome.card, true)
                        is CardMenuOutcome.SavePicture -> pictureSaved(outcome.card)
                        is CardMenuOutcome.OpenUrl -> runCatching { uriHandler.openUri(outcome.url) }
                        is CardMenuOutcome.Deleted, is CardMenuOutcome.RemovedFromSpace -> {
                            if (outcome is CardMenuOutcome.Deleted) recent.removeAll { it.id == outcome.cardId }
                            if (opened?.card?.id == target.card.id) dropCard()
                        }
                        is CardMenuOutcome.PinChanged -> updateCard(outcome.card)
                        is CardMenuOutcome.Rotated -> updateCard(outcome.card)
                        is CardMenuOutcome.Downloaded -> Toast.makeText(context, DOWNLOADED, Toast.LENGTH_SHORT).show()
                        is CardMenuOutcome.Read -> read(outcome.card, target.spaceId)
                        is CardMenuOutcome.ShowSimilar -> {
                            dropCard()
                            // The similar cards are a search, so they show on Ask.
                            askFocus = false
                            searchActive = true
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
            root = sheetRoot,
            rootContent = when (val root = sheetRoot) {
                CardSheetRoot.Share -> ({
                    val card = target.card
                    ShareContent(
                        card,
                        onShareLink = card.url?.let { url -> { closeMenu(); send(url, card.title) } },
                        onShareText = shareText(card)?.let { text -> { closeMenu(); send(text, card.title) } },
                        onCopyLink = card.url?.let { { closeMenu(); copyLink(card) } },
                        onSharePicture = galleryPicture(card)?.let { { closeMenu(); pictureOut(card, false) } },
                        onCardImage = { submenu = CardSubmenu.CardImage },
                        onCopyAgentLink = { closeMenu(); copyAgentLink(card) },
                    )
                })
                is CardSheetRoot.Part -> ({
                    val card = target.card
                    PartContent(root.part, card) { choice ->
                        when (choice.action) {
                            PartAction.Copy -> partText(root.part, card)?.let { text ->
                                closeMenu()
                                scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText(root.part.heading, text))) }
                            }
                            PartAction.Share -> partText(root.part, card)?.let { text -> closeMenu(); send(text, card.title) }
                            PartAction.EditTags -> if (opened?.card?.id == card.id && card.tags.orEmpty().any { !it.name.isNullOrEmpty() }) {
                                closeMenu()
                                opened = opened?.copy(editingTags = true)
                            } else {
                                viewModel.loadTagChoices()
                                submenu = CardSubmenu.Tags
                            }
                            PartAction.EditNote -> submenu = CardSubmenu.Note
                            PartAction.Read -> {
                                closeMenu()
                                read(card, target.spaceId)
                            }
                            PartAction.SavePicture -> {
                                closeMenu()
                                // A picture with a file of its own saves that file; any other picture is saved as drawn.
                                if (card.blob?.path == null || card.blob.type?.startsWith("image/") != true) pictureSaved(card)
                                else scope.launch {
                                    when (val outcome = viewModel.perform(CardMenuItem(CardMenuAction.Download), card, target.spaceId)) {
                                        is CardMenuOutcome.Downloaded -> Toast.makeText(context, DOWNLOADED, Toast.LENGTH_SHORT).show()
                                        is CardMenuOutcome.Failed -> failure = outcome.failure
                                        else -> Unit
                                    }
                                }
                            }
                            PartAction.CopyPicture -> {
                                closeMenu()
                                pictureOut(card, true)
                            }
                            PartAction.SharePicture -> {
                                closeMenu()
                                pictureOut(card, false)
                            }
                        }
                    }
                })
                CardSheetRoot.Actions -> null
            },
            spaces = {
                InlineSpaces(choices, target.card, onToggle = toggleSpace, onAll = {
                    viewModel.acknowledgeCardSpaceCreation()
                    submenu = CardSubmenu.Spaces
                })
            },
            onDeleteHold = { deleteHold = it },
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
                        CardSubmenu.Spaces -> AllSpacesPage(
                            choices = choices,
                            creation = cardSpaceCreation,
                            onToggle = toggleSpace,
                            onCreate = { name, color -> viewModel.createCardSpace(name, color, menuFor?.card) },
                            onAcknowledge = viewModel::acknowledgeCardSpaceCreation,
                        )
                        CardSubmenu.Note -> NotePage(target.card.note.orEmpty(), onSave = { text ->
                            focusManager.clearFocus()
                            closeMenu()
                            scope.launch {
                                val card = menuFor?.card ?: return@launch
                                when (val answer = viewModel.setNote(card, text)) {
                                    is ApiResult.Ok -> updateCard(answer.value)
                                    is ApiResult.Failed -> failure = answer.failure
                                }
                            }
                        })
                        CardSubmenu.CardImage -> CardImagePage(target.card) { image ->
                            val card = target.card
                            closeMenu()
                            scope.launch {
                                val uri = cacheImage(context, image.asAndroidBitmap(), sharedImageName(card, ".png"), png = true)
                                sharePicture(context, uri, "image/png")
                            }
                        }
                    }
            },
        )
        }
    }
    // Unfolded, the sheets opened from the gallery rise over the gallery's pane.
    CompositionLocalProvider(LocalPaneSpan provides if (unfolded) panes.list.value else null) {
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
        )
    }
    when (packSheet) {
        PackSheet.Tags -> ChoiceTagSheet(choice.cards, tagChoices.recent + tagChoices.all,
            onDone = { put, takeOff ->
                packSheet = null
                if (put.isNotEmpty() || takeOff.isNotEmpty()) scope.launch {
                    val change = viewModel.tagMany(choice.cards, put, takeOff)
                    choice.refresh(change.cards)
                    manyDone = manyText(if (put.isNotEmpty()) "Tagged" else "Took tags off", change) to change
                }
            },
            onDismiss = { packSheet = null })
        PackSheet.Spaces -> ChoiceSpaceSheet(choice.count, spaces.spaces,
            onPick = { space ->
                packSheet = null
                scope.launch {
                    val change = viewModel.putManyInSpace(choice.cards, space)
                    choice.refresh(change.cards)
                    manyDone = manyText("Put", change, " in ${space.name ?: "the space"}") to change
                }
            },
            onDismiss = { packSheet = null })
        PackSheet.Copy -> ChoiceCopySheet(choice.cards,
            onPick = { pick ->
                packSheet = null
                packOut(pick, choice.cards.toList())
            },
            onDismiss = { packSheet = null })
        null -> Unit
    }
    if (profileOpen) ProfileSheetHost(settingsViewModel, updates, onClose = { profileOpen = false }, ai = ai, feedback = feedback, photoLoader = photoLoader,
        pageScheme = scheme)
    if (newSpaceOpen) {
        PageTheme(scheme) {
            NewSpaceSheet(
                busy = creatingSpace,
                error = newSpaceError,
                onClose = { newSpaceOpen = false },
                onCreate = { name, color ->
                    creatingSpace = true
                    newSpaceError = null
                    scope.launch {
                        when (val answer = viewModel.createSpace(name, color)) {
                            is ApiResult.Ok -> newSpaceOpen = false
                            is ApiResult.Failed -> newSpaceError = describe(answer.failure)
                        }
                        creatingSpace = false
                    }
                },
            )
        }
    }
    }
    extras()
    failure?.let { ApiErrorDialog(title = null, message = describe(it), onDismiss = { failure = null }) }
}

private const val DOWNLOADED = "Successfully downloaded"

/** The folder of the phone's cache that keeps opened PDFs. */
private const val PDF_DIR = "pdf"
private const val NO_PICTURE = "The picture could not be loaded"

/** How many recently opened cards the shell keeps for the reader's name card. */
private const val RECENT_KEPT = 8

/**
 * The frame of the signed-in screens: the page between the status bar and the tab bar in the account's appearance,
 * with the status bar background and the tab bar in the system's.
 *
 * @param tab Tab shown as selected.
 * @param onSelectTab Called with the tab the user tapped.
 * @param scheme Appearance the account picked for the pages.
 * @param modifier Modifier for the full-screen box.
 * @param recede How far an opened reader's sheet has risen over the page, from 0 to 1, read while drawing: the page and
 *   its bar step back into a smaller, dimmed card with round corners, on black.
 * @param overlay Drawn over the page and the tab bar, such as an opened card.
 * @param beside On an unfolded window, the pane beside the gallery, such as an opened card or the line saying what goes
 *   there (Pully's rule 206); null, or any other window, shows the page alone.
 * @param panes Where the two panes sit once [beside] shows, for the sheets that rise over them.
 * @param page The selected tab's page.
 */
@Composable
fun ShellFrame(
    tab: Tab,
    onSelectTab: (Tab) -> Unit,
    scheme: PageScheme,
    modifier: Modifier = Modifier,
    footer: (@Composable () -> Unit)? = null,
    recede: () -> Float = { 0f },
    overlay: @Composable BoxScope.() -> Unit = {},
    beside: (@Composable () -> Unit)? = null,
    panes: ShellPanes = remember { ShellPanes() },
    page: @Composable () -> Unit,
) {
    val bars = remember { FootBars() }
    val immersion = remember { ShellImmersion() }
    CompositionLocalProvider(LocalFootBars provides bars, LocalShellImmersion provides immersion) {
        val paper = MymindTheme.colors.page
        val scrim = MymindTheme.palette.sheet.scrim
        val top = WindowInsets.statusBars.getTop(LocalDensity.current).toFloat()
        ImmersiveSystemBars(immersion)
        val shape = rememberWindowShape()
        val library: @Composable BoxScope.() -> Unit = {
            PageTheme(scheme) {
                Box(Modifier.fillMaxSize().themeBackground(LibraryStyle.colors.background)
                    .drawBehind { immersion.strip?.invoke()?.let { drawRect(it, size = Size(size.width, top)) } }.statusBarsPadding()) {
                    // Sideways, the page sits in a readable column with the page colour either side (Pully's rule 207).
                    Box(if (shape == WindowShape.Sideways) Modifier.fillMaxHeight().readableWidth() else Modifier.fillMaxSize()) { page() }
                }
                if (footer != null) footer() else MymindTabBar(tab, onSelectTab)
                // A full-screen page slides the bar down out of sight, and out of reach.
                FootBarLayer(bars, Modifier.align(Alignment.BottomCenter).graphicsLayer {
                    val away = immersion.amount()
                    alpha = 1f - away
                    translationY = away * size.height
                })
            }
        }
        if (shape == WindowShape.Unfolded && beside != null) {
            // Unfolded, the gallery and the card it opened sit side by side, each with its own foot (Pully's rule 206).
            BoxWithConstraints(modifier.fillMaxSize().drawBehind { drawRect(lerp(paper, Color.Black, immersion.amount())) }) {
                val listWidth = listPaneWidth(maxWidth)
                Row(Modifier.fillMaxSize().windowInsetsPadding(SideInsets)) {
                    Box(Modifier.width(listWidth).fillMaxHeight().spanInto(panes.list)) {
                        CompositionLocalProvider(LocalPaneSpan provides panes.list.value) { library() }
                    }
                    Box(Modifier.width(1.dp).fillMaxHeight().background(LibraryStyle.colors.muted.copy(alpha = HAIRLINE_ALPHA)))
                    Box(Modifier.weight(1f).fillMaxHeight().background(LibraryStyle.colors.background).spanInto(panes.beside)) {
                        PageTheme(scheme) { beside() }
                    }
                }
                overlay()
            }
            return@CompositionLocalProvider
        }
        Box(modifier.fillMaxSize().drawBehind { drawRect(lerp(lerp(paper, Color.Black, immersion.amount()), Color.Black, recede())) }) {
            Box(Modifier.fillMaxSize().stepBack(recede, top, scrim).drawBehind { drawRect(lerp(paper, Color.Black, immersion.amount())) }
                .windowInsetsPadding(SideInsets), content = library)
            overlay()
        }
    }
}

/** How strong the hairline between the gallery and the card beside it is, over the muted ink. */
private const val HAIRLINE_ALPHA = 0.25f

/**
 * Where the two panes of an unfolded window sit, so a sheet rises over the pane whose page opened it (Pully's rule 208).
 *
 * @property list The gallery's pane, or null until laid out or on any other window.
 * @property beside The pane beside it.
 */
@Stable
class ShellPanes {
    val list = mutableStateOf<PaneSpan?>(null)
    val beside = mutableStateOf<PaneSpan?>(null)
}

/** Keeps [span] on where this pane sits in the window. */
private fun Modifier.spanInto(span: MutableState<PaneSpan?>): Modifier = onGloballyPositioned {
    val next = PaneSpan(it.positionInWindow().x, it.size.width.toFloat())
    val now = span.value
    if (now == null || now.left != next.left || now.width != next.width) span.value = next
}

/**
 * The pane beside the gallery while no card is open in it: one quiet line saying what goes there.
 *
 * @param modifier Modifier for the pane.
 */
@Composable
internal fun NothingOpen(modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize().padding(horizontal = 32.dp), contentAlignment = Alignment.Center) {
        Text(NOTHING_OPEN, style = LibraryStyle.body, color = LibraryStyle.colors.muted, textAlign = TextAlign.Center)
    }
}

private const val NOTHING_OPEN = "A card you open shows here."

/** Which of the backpack's sheets is open. */
private enum class PackSheet { Tags, Spaces, Copy }

/** How much higher a batch's Undo sits while the backpack's bar is showing under it. */
private val PACK_BAR_HEIGHT = 80.dp

/**
 * What a batch's Undo bar says: "Tagged 12 finds", with how many could not be changed when some could not.
 *
 * @param verb What was done.
 * @param change What the batch changed.
 * @param after Words after the count, such as " in Travel".
 */
private fun manyText(verb: String, change: ManyChange, after: String = ""): String {
    val text = "$verb ${findsLabel(change.chosen - change.failed)}$after"
    return if (change.failed == 0) text else "$text, ${change.failed} could not be changed"
}

/**
 * Steps the page back behind a reader's sheet as far as [recede] says: smaller, its top [top] px plus a little below the
 * screen's top so its edge shows above the sheet, its corners as round as the sheet's on screen, and under [scrim].
 */
private fun Modifier.stepBack(recede: () -> Float, top: Float, scrim: Color): Modifier = graphicsLayer {
    val r = recede()
    if (r <= 0f) return@graphicsLayer
    val scale = 1f - ReaderSheetLayout.GALLERY_SHRINK * r
    scaleX = scale
    scaleY = scale
    transformOrigin = TransformOrigin(0.5f, 0f)
    translationY = (top + ReaderSheetLayout.GalleryGap.toPx()) * r
    shape = RoundedCornerShape(ReaderSheetLayout.Corner * r / scale)
    clip = true
}.drawWithContent {
    drawContent()
    val r = recede()
    if (r > 0f) drawRect(scrim, alpha = r)
}

/** The name usage events give the shell's page [page]. */
private fun screenName(page: Int): String = if (page == ASK_PAGE) "ask" else Tab.entries[page].name.lowercase()
