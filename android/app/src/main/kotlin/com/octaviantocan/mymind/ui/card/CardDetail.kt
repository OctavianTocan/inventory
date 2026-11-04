package com.octaviantocan.mymind.ui.card

import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.annotation.DrawableRes
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.positionInWindow
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.compose.ui.zIndex
import androidx.core.view.WindowCompat
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.PostArticle
import com.octaviantocan.mymind.data.RedditComment
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.CardPicture
import com.octaviantocan.mymind.ui.library.CardThumbnail
import com.octaviantocan.mymind.ui.library.LocalVideoPreviews
import com.octaviantocan.mymind.ui.library.PostBrand
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.postArticle
import com.octaviantocan.mymind.ui.library.postBrand
import com.octaviantocan.mymind.ui.library.postByline
import com.octaviantocan.mymind.ui.library.postWords
import com.octaviantocan.mymind.ui.library.productPrice
import com.octaviantocan.mymind.ui.library.productShop
import com.octaviantocan.mymind.ui.library.readerVideo
import com.octaviantocan.mymind.ui.library.redditComments
import com.octaviantocan.mymind.ui.library.tileKind
import com.octaviantocan.mymind.ui.theme.MymindTheme

private val BODY_SIDE = 16.dp

/** Room the page leaves under its last section, so the foot's buttons and fade never cover it. */
private val FOOT_ROOM = 112.dp

/** The parts shown above the tags; the user's note and the page text follow them. */
private val LEAD_PARTS = setOf(ReaderPart.Post, ReaderPart.Note, ReaderPart.Summary)

/**
 * What the reader's controls do besides closing, opening the source and opening the card's actions. A null action
 * hides its control.
 *
 * @property onShare Opens the Share sheet from the header.
 * @property onAddTags Opens Add tags from the Tags section.
 * @property onCopyLink Puts the card's link on the clipboard, from the name's card.
 * @property onTogglePin Pins the card to Top of Mind or takes it off, from the name's card.
 * @property onSpaces Opens the card's spaces, from the name's card.
 * @property onOpenRecent Opens a recently opened card in place of this one.
 * @property onEverything Leaves the reader for the Everything gallery.
 * @property onPart Opens the sheet of a part that was held; null leaves the parts without one.
 * @property onReading Turns Reader Mode on or off.
 * @property onEditingTags Shows the crosses that take tags off, or hides them again with Done.
 */
@Immutable
class ReaderActions(
    val onShare: (() -> Unit)? = null,
    val onAddTags: (() -> Unit)? = null,
    val onCopyLink: (() -> Unit)? = null,
    val onTogglePin: (() -> Unit)? = null,
    val onSpaces: (() -> Unit)? = null,
    val onOpenRecent: ((Card) -> Unit)? = null,
    val onEverything: (() -> Unit)? = null,
    val onPart: ((ReaderPart) -> Unit)? = null,
    val onReading: ((Boolean) -> Unit)? = null,
    val onEditingTags: ((Boolean) -> Unit)? = null,
)

/**
 * What the reader shows about a card that the card itself does not hold.
 *
 * @property spaces Names of the spaces the card is in, in the user's order.
 * @property recent Cards opened before this one, latest first.
 * @property sameVibe The cards nearest a product in meaning, nearest first, for its Same Vibe section.
 */
@Immutable
data class ReaderDetails(val spaces: List<String> = emptyList(), val recent: List<Card> = emptyList(), val sameVibe: List<Card> = emptyList())

/**
 * A card reader in its own window, as a sheet that rises over the gallery and rests just under the status bar;
 * contextual menu windows opened afterward remain above it. A finger pulls it down from its header, or from the top of
 * its content, and letting it go far enough or throwing it down closes it.
 *
 * @param onClose Called on Back; the caller sinks [sheet] and then removes the reader.
 * @param actionsProgress How far the card's sheet has risen over it, from 0 to 1; the reader blurs with it.
 * @param shrink How far Delete card's hold has come, from 0 to 1; the reader shrinks and fades with it.
 * @param reading True for Reader Mode: the page's text alone, in larger type.
 * @param editingTags True while the tags show the crosses that take them off, after Edit tags from their held sheet.
 * @param sheet Where the sheet is; the default rests in place, as a reader shown on its own does.
 * @param onPulledAway Called once a finger has taken the sheet off the bottom of the screen, to remove the reader.
 */
@Composable
fun CardDetailDialog(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    onRemoveTag: ((Tag) -> Unit)? = null,
    actionsProgress: Float = 0f,
    shrink: Float = 0f,
    reading: Boolean = false,
    editingTags: Boolean = false,
    scrollState: ScrollState = rememberScrollState(),
    actions: ReaderActions = ReaderActions(),
    details: ReaderDetails = ReaderDetails(),
    sheet: ReaderSheetState = remember { ReaderSheetState(risen = true) },
    onPulledAway: () -> Unit = onClose,
) {
    Dialog(
        onDismissRequest = onClose,
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val view = LocalView.current
        val light = MymindTheme.page.document.luminance() > 0.5f
        // Behind the status bar is the black the gallery stepped back onto, until a finger brings the gallery back.
        val overBlack by remember(sheet) { derivedStateOf { sheet.recede() > 0.5f } }
        SideEffect {
            val window = (view.parent as? DialogWindowProvider)?.window ?: return@SideEffect
            window.setDimAmount(0f)
            window.setWindowAnimations(0)
            window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
            window.isNavigationBarContrastEnforced = false
            WindowCompat.getInsetsController(window, view).apply {
                isAppearanceLightStatusBars = light && !overBlack
                isAppearanceLightNavigationBars = light
            }
        }
        val density = LocalDensity.current
        val scope = rememberCoroutineScope()
        val motion = rememberSheetMotionEnabled()
        val latestMotion by rememberUpdatedState(motion)
        val latestGone by rememberUpdatedState(onPulledAway)
        val speed = with(density) { THROW_SPEED.toPx() }
        val connection = remember(sheet, scope, speed) { ReaderSheetScroll(sheet, scope, speed, { latestMotion }, { latestGone() }) }
        val top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding() + ReaderSheetLayout.SheetGap
        val corner = RoundedCornerShape(topStart = ReaderSheetLayout.Corner, topEnd = ReaderSheetLayout.Corner)
        Box(Modifier.fillMaxSize()) {
            Box(Modifier.fillMaxSize().padding(top = top)
                .consumeWindowInsets(WindowInsets.safeDrawing.only(WindowInsetsSides.Top))
                .onSizeChanged { sheet.height = it.height.toFloat() }
                .graphicsLayer {
                    val rise = sheet.rise
                    translationY = (1f - rise) * size.height * RISE_FROM + sheet.pull
                    alpha = rise
                    val blur = SHEET_BLUR.toPx() * (1f - rise)
                    renderEffect = if (blur > 0.1f) BlurEffect(blur, blur, TileMode.Decal) else null
                    shape = corner
                    clip = true
                }
                .nestedScroll(connection)
                .draggable(rememberDraggableState { sheet.drag(it) }, Orientation.Vertical,
                    onDragStopped = { velocity -> sheet.release(scope, velocity, speed, latestMotion) { latestGone() } })) {
                CardDetail(card, onClose, onMenu, modifier = Modifier.blur(8.dp * actionsProgress).shrunk { shrink }, onRemoveTag = onRemoveTag,
                    scrollState = scrollState, actions = actions, details = details, reading = reading,
                    editingTags = editingTags)
                Box(Modifier.align(Alignment.TopCenter).padding(top = 8.dp).size(36.dp, 4.dp).clip(CircleShape)
                    .background((if (light) LightCardSheetColors else DarkCardSheetColors).grabber))
            }
        }
    }
}

/** How far below its rest the sheet starts rising, as a share of its height (transitions.dev's Panel reveal: half). */
private const val RISE_FROM = 0.5f

/** How blurred the sheet is as it starts rising and as it finishes sinking. */
private val SHEET_BLUR = 2.dp

/** A throw faster than this closes the sheet however short it was, as on the card's sheets. */
private val THROW_SPEED = 200.dp

/**
 * Shrinks and fades the page as Delete card is held, so the card visibly goes.
 *
 * @param progress How far the hold has come, from 0 to 1.
 */
fun Modifier.shrunk(progress: () -> Float): Modifier = graphicsLayer {
    val held = progress()
    scaleX = 1f - DELETE_SHRINK * held
    scaleY = 1f - DELETE_SHRINK * held
    alpha = 1f - DELETE_FADE * held
}

private const val DELETE_SHRINK = 0.1f
private const val DELETE_FADE = 0.45f

/**
 * A fixed, full-height reading page: Back, the card's name and Share above the card's parts in raised sections, with
 * Open source and the card's actions at the foot. The name grows into a card of its details and recently opened cards.
 * The card's type colour washes the top edge, as it does on the card's sheets. Holding a part opens its own sheet;
 * Reader Mode swaps the parts for the page's text alone, with a line in the name's pill for how far it has been read.
 */
@Composable
internal fun CardDetail(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    modifier: Modifier = Modifier,
    scrollState: ScrollState = rememberScrollState(),
    headerIcons: CardHeaderIcons = defaultCardHeaderIcons(),
    onRemoveTag: ((Tag) -> Unit)? = null,
    onOpenSource: ((String) -> Unit)? = null,
    actions: ReaderActions = ReaderActions(),
    details: ReaderDetails = ReaderDetails(),
    reading: Boolean = false,
    editingTags: Boolean = false,
) {
    val colors = if (MymindTheme.page.document.luminance() < 0.5f) DarkCardSheetColors else LightCardSheetColors
    val density = LocalDensity.current
    val uriHandler = LocalUriHandler.current
    val open = onOpenSource ?: { url: String -> runCatching { uriHandler.openUri(url) }; Unit }
    val source = sourceUrl(card)?.let { url -> { open(url) } }
    val context = LocalContext.current
    val pdf = rememberPdfPages(card)
    var pdfText by remember(pdf) { mutableStateOf<String?>(null) }
    LaunchedEffect(pdf) { pdfText = pdf?.text() }
    val paragraphs = remember(card, pdfText) { readerParagraphs(card).ifEmpty { pdfText?.lines()?.filter { it.isNotBlank() }.orEmpty() } }
    val viewPdf = pdf?.let { pages -> { pages.view(context) } }
    val wash = cardAccent(card)?.copy(alpha = if (colors.paper.luminance() < 0.5f) 0.16f else 0.12f)
    var named by rememberSaveable(card.id) { mutableStateOf(false) }
    val readingScroll = rememberScrollState()
    val inReader = reading && paragraphs.isNotEmpty()
    val back = if (inReader) ({ actions.onReading?.invoke(false); Unit }) else onClose
    BackHandler(onBack = back)
    CompositionLocalProvider(LocalCardSheetColors provides colors) {
        Box(modifier.fillMaxSize().background(colors.paper)
            .drawBehind {
                if (wash != null) drawRect(Brush.verticalGradient(listOf(wash, Color.Transparent), endY = 220.dp.toPx()))
            }
            .semantics {
                paneTitle = "Card details"
                dismiss { onClose(); true }
            }) {
            Column(Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing)) {
                Spacer(Modifier.height(16.dp))
                ReaderHeader(cardHeading(card), named, onBack = back, onName = { named = !named }, onShare = actions.onShare, icons = headerIcons,
                    progress = if (inReader) ({ readingScroll.value / readingScroll.maxValue.coerceAtLeast(1).toFloat() }) else null)
                Box(Modifier.fillMaxWidth().weight(1f)) {
                    val fade = with(density) { 20.dp.toPx() }
                    val shown = if (inReader) readingScroll else scrollState
                    Column(Modifier.fillMaxSize()
                        .sheetEdgeFade(color = colors.paper, topStrength = { (shown.value / fade).coerceIn(0f, 1f) })
                        .verticalScroll(shown)) {
                        if (inReader) ReaderModeBody(card, paragraphs, onBack = back)
                        else ReaderBody(card, onRemoveTag?.takeIf { editingTags }, actions.onEditingTags?.let { set -> { set(false) } },
                            actions.onAddTags, source, onMenu, actions.onPart, details.sameVibe, actions.onOpenRecent,
                            pdf, pdfText?.let(::readingMinutes), actions.onReading?.takeIf { paragraphs.isNotEmpty() }?.let { read -> { read(true) } })
                    }
                    if (!inReader) ReaderFoot(card, source ?: viewPdf, onMenu, Modifier.align(Alignment.BottomCenter),
                        openLabel = if (source == null && viewPdf != null) "Open the PDF" else null)
                }
            }
            ReaderNameCard(named, card, details, actions, onClose = { named = false })
        }
    }
}

/**
 * The reader's one-row header: Back, the card's name in a raised pill that opens its details, and Share.
 *
 * @param title The card's name, kept whole in the pill's text even when it ellipsizes.
 * @param open True while the name's card is showing; the pill's caret turns up.
 * @param onBack Returns to the gallery.
 * @param onName Shows or hides the name's card.
 * @param onShare Opens the Share sheet; null leaves the place empty.
 * @param modifier Modifier for the row.
 * @param icons Back and share glyphs.
 * @param progress How far Reader Mode has been read, from 0 to 1, drawn as a line in the pill; null draws none.
 */
@Composable
internal fun ReaderHeader(
    title: String,
    open: Boolean,
    onBack: () -> Unit,
    onName: () -> Unit,
    onShare: (() -> Unit)?,
    modifier: Modifier = Modifier,
    icons: CardHeaderIcons = defaultCardHeaderIcons(),
    progress: (() -> Float)? = null,
) {
    Row(modifier.fillMaxWidth().zIndex(1f).padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        // The reader returns to the gallery, so it leads with Back rather than a closing cross.
        CardHeaderButton(icons.back ?: painterResource(R.drawable.ic_sheet_back), "Back", onBack, true)
        NamePill(title, open, onName, Modifier.weight(1f), progress)
        if (onShare != null) CardHeaderButton(icons.share ?: painterResource(R.drawable.ic_share), "Share", onShare, true)
        else Spacer(Modifier.size(44.dp))
    }
}

/**
 * The card's name on one line with a caret, in a raised pill; tapping it shows or hides the name's card. In Reader Mode
 * a thin line along its foot fills as the page is read.
 */
@Composable
private fun NamePill(title: String, open: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier, progress: (() -> Float)? = null) {
    val colors = LocalCardSheetColors.current
    val shape = RoundedCornerShape(50)
    val haptic = rememberSheetHaptics()
    val motion = rememberSheetMotionEnabled()
    val turn by animateFloatAsState(if (open) 180f else 0f, tween(if (motion) CardSheetMotion.FAST else 0, easing = CardSheetMotion.SmoothOut), label = "nameCaret")
    Row(
        modifier.heightIn(min = 44.dp).cardSurfaceShadow(shape).clip(shape).background(colors.raised)
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button,
                onClickLabel = if (open) "Hide card details" else "Show card details") { haptic.tick(); onClick() }
            .semantics { stateDescription = if (open) "Details shown" else "Details hidden" }
            .then(if (progress == null) Modifier else Modifier.drawBehind {
                val inset = 20.dp.toPx()
                val line = 3.dp.toPx()
                val y = size.height - 5.dp.toPx() - line
                val width = size.width - inset * 2
                drawRoundRect(colors.active, Offset(inset, y), Size(width, line), CornerRadius(line / 2))
                drawRoundRect(colors.text, Offset(inset, y), Size(width * progress().coerceIn(0f, 1f), line), CornerRadius(line / 2))
            })
            .padding(horizontal = 16.dp, vertical = 9.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, Modifier.weight(1f, fill = false), style = readerTitleStyle(colors), maxLines = 1, overflow = TextOverflow.Ellipsis)
        Image(painterResource(R.drawable.ic_caret_down), null, Modifier.size(16.dp).rotate(turn),
            colorFilter = ColorFilter.tint(colors.text.copy(alpha = 0.65f)))
    }
}

/** Pully's sheet title, which the name's pill and card share: 18/26 semibold. */
@Composable
internal fun readerTitleStyle(colors: CardSheetColors) = TextStyle(
    fontFamily = MymindTheme.fonts.heading, fontWeight = FontWeight.SemiBold, fontSize = 18.sp, lineHeight = 26.sp, color = colors.text,
)

/**
 * The card's parts, each in its own raised section: picture and title, its words, tags, the user's note and the page.
 * With [onPart], each part but the picture opens its own sheet when held, a card that is not a note always shows Your
 * note so one can be written.
 */
@Composable
private fun ReaderBody(
    card: Card,
    onRemoveTag: ((Tag) -> Unit)?,
    onDoneTags: (() -> Unit)?,
    onAddTags: (() -> Unit)?,
    onOpenSource: (() -> Unit)?,
    onMenu: () -> Unit,
    onPart: ((ReaderPart) -> Unit)?,
    sameVibe: List<Card> = emptyList(),
    onOpenCard: ((Card) -> Unit)? = null,
    pdf: PdfPages? = null,
    pdfMinutes: Int? = null,
    onRead: (() -> Unit)? = null,
) {
    val sections = readerSections(card)
    val hold = { part: ReaderPart -> onPart?.takeIf { partChoices(part, card).isNotEmpty() }?.let { open -> { open(part) } } }
    val pictureHold = hold(ReaderPart.Picture)
    val emptyNote = onPart != null && tileKind(card) != TileKind.Note && sections.none { it.part == ReaderPart.YourNote }
    val post = tileKind(card) == TileKind.Post
    val document = isPdf(card)
    Column(Modifier.fillMaxWidth().padding(horizontal = BODY_SIDE).padding(top = 12.dp, bottom = FOOT_ROOM),
        verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (post) PostSection(card, onOpenSource, pictureHold ?: onMenu, if (pictureHold != null) "Picture actions" else "Card actions", hold(ReaderPart.Post))
        else if (document) PdfHeadSection(card, pdf?.count, pdfMinutes, onRead)
        else HeadSection(card, onOpenSource, pictureHold ?: onMenu, if (pictureHold != null) "Picture actions" else "Card actions")
        if (document) PdfPagesSection(card, pdf, onMenu)
        val comments = redditComments(card)
        if (comments.isNotEmpty()) CommentsSection(comments, onMenu)
        sections.filter { it.part in LEAD_PARTS && !(post && it.part == ReaderPart.Post) }.forEach { TextSection(it, card, hold(it.part)) }
        if (sameVibe.isNotEmpty()) SameVibeSection(sameVibe, onOpenCard)
        TagsSection(card, onRemoveTag, onDoneTags, onAddTags, hold(ReaderPart.Tags))
        sections.filter { it.part == ReaderPart.YourNote }.forEach { TextSection(it, card, hold(it.part)) }
        if (emptyNote) EmptyNoteSection(hold(ReaderPart.YourNote))
        sections.filter { it.part == ReaderPart.Page }.forEach { TextSection(it, card, hold(it.part)) }
    }
}

/** A raised reading section with Pully's note shadow and 24dp corners; [onHold] opens its part's sheet when it is held. */
@Composable
private fun ReaderSection(
    padding: PaddingValues,
    modifier: Modifier = Modifier,
    holdLabel: String? = null,
    onHold: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = RoundedCornerShape(24.dp)
    Column(modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(LocalCardSheetColors.current.raised)
        .held(holdLabel, onHold).padding(padding), content = content)
}

/**
 * Opens a part's sheet from a long press, with a haptic tick, and offers the same as an accessibility action.
 *
 * @param label What the action says to accessibility services.
 * @param onHold What holding does; null leaves the modifier unchanged.
 */
@Composable
private fun Modifier.held(label: String?, onHold: (() -> Unit)?): Modifier {
    if (onHold == null) return this
    val haptic = LocalHapticFeedback.current
    val hold by rememberUpdatedState(onHold)
    return this
        .pointerInput(Unit) {
            detectTapGestures(onLongPress = {
                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                hold()
            })
        }
        .semantics { onLongClick(label) { hold(); true } }
}

/** Your note before one is written: a quiet invitation to hold it and write one. */
@Composable
private fun EmptyNoteSection(onHold: (() -> Unit)?) {
    val colors = LocalCardSheetColors.current
    ReaderSection(SECTION_PADDING, holdLabel = "Write a note", onHold = onHold) {
        SectionHeading(partIcon(ReaderPart.YourNote), ReaderPart.YourNote.heading)
        Spacer(Modifier.height(8.dp))
        Text("No note yet. Hold here to write one.", style = detailStyle(colors).copy(fontFamily = MymindTheme.fonts.body))
    }
}

/**
 * Reader Mode: the card's title and byline over the page's text alone, in larger type that can be selected, ending
 * with Back to the card.
 */
@Composable
private fun ReaderModeBody(card: Card, paragraphs: List<String>, onBack: () -> Unit) {
    val colors = LocalCardSheetColors.current
    val minutes = paragraphs.takeIf { it.isNotEmpty() }?.let { "${readingMinutes(it.joinToString(" "))} min read" }
    val byline = listOfNotNull(bylineOf(card), minutes).joinToString(" · ").ifEmpty { null }
    Column(Modifier.fillMaxWidth().padding(start = 24.dp, end = 24.dp, top = 24.dp, bottom = 48.dp)) {
        Text(cardHeading(card), Modifier.semantics { heading() }, style = TextStyle(fontFamily = MymindTheme.fonts.heading,
            fontWeight = FontWeight.SemiBold, fontSize = 28.sp, lineHeight = 34.sp, color = colors.text))
        if (byline != null) Text(byline, Modifier.padding(top = 6.dp), style = detailStyle(colors).copy(fontFamily = MymindTheme.fonts.body))
        Spacer(Modifier.height(22.dp))
        SelectionContainer {
            Column(verticalArrangement = Arrangement.spacedBy(18.dp)) {
                paragraphs.forEach {
                    Text(it, style = TextStyle(color = colors.text, fontSize = 19.sp, lineHeight = 30.sp, fontFamily = MymindTheme.fonts.body))
                }
            }
        }
        val shape = RoundedCornerShape(50)
        Row(Modifier.padding(top = 30.dp).align(Alignment.CenterHorizontally).heightIn(min = 44.dp).clip(shape).background(colors.active)
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onBack)
                .padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Image(painterResource(R.drawable.ic_sheet_back), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.text))
            Text("Back to the card", style = TextStyle(color = colors.text, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = MymindTheme.fonts.body))
        }
    }
}

/**
 * The picture, or the video the server keeps, over the title, who made the card and when it was saved. A tap on the
 * picture opens the source and holding it calls [onHold], which says [holdLabel]. A card with neither picture nor
 * title has none.
 */
@Composable
private fun HeadSection(card: Card, onOpenSource: (() -> Unit)?, onHold: () -> Unit, holdLabel: String) {
    val colors = LocalCardSheetColors.current
    val previews = LocalVideoPreviews.current
    val video = remember(card) { readerVideo(card) }?.takeIf { previews != null }
    val picture = galleryPicture(card) != null
    val title = card.title?.trim()?.takeIf { it.isNotEmpty() }
    val price = if (card.type == "Product") productPrice(card) else null
    val by = if (card.type == "Product") listOfNotNull(productShop(card), siteOf(card)).distinct().joinToString(" · ").ifEmpty { null } else bylineOf(card)
    val lines = listOfNotNull(by, listOfNotNull(savedLabel(card.created), readingTime(card)).joinToString(" · ").ifEmpty { null })
    val media = video != null || picture
    if (!media && title == null) return
    ReaderSection(PaddingValues(8.dp)) {
        val frame = Modifier.fillMaxWidth().clip(RoundedCornerShape(18.dp))
        if (video != null && previews != null) CardVideo(card, video, previews, frame)
        else if (picture) Box(frame) {
            CardPicture(card, play = tileKind(card) == TileKind.Video, whole = true, modifier = Modifier
                .semantics { contentDescription = "Preview" }
                .combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, role = Role.Button,
                    onClickLabel = if (onOpenSource != null) "Open source" else null, onClick = { onOpenSource?.invoke() },
                    onLongClickLabel = holdLabel, onLongClick = onHold,
                ))
            if (price != null) Text(price, Modifier.align(Alignment.BottomStart).padding(12.dp).shadow(3.dp, CircleShape).clip(CircleShape)
                .background(Color.White).padding(horizontal = 14.dp, vertical = 7.dp),
                style = TextStyle(color = Color(0xFF262520), fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium,
                    fontFamily = MymindTheme.fonts.body))
        }
        if (title != null || lines.isNotEmpty()) {
            Column(Modifier.fillMaxWidth().padding(start = 8.dp, end = 8.dp, top = if (media) 12.dp else 8.dp, bottom = 10.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)) {
                if (title != null) Text(title, style = TextStyle(fontFamily = MymindTheme.fonts.heading, fontWeight = FontWeight.SemiBold,
                    fontSize = 26.sp, lineHeight = 32.sp, color = colors.text))
                lines.forEach { Text(it, style = detailStyle(colors)) }
            }
        }
    }
}

/**
 * A social post as one section in its network's own colours ([postBrand]): who posted it, with the handle and the day
 * and the network's mark; a Reddit post's title; its words; its picture or the video the server keeps, whole; then when
 * it was posted and saved. It is ThyMind's own copy, never a live embed, and shows no counts. Holding the words opens
 * the post's sheet ([onHold]); the picture opens the source when tapped and calls [onPictureHold] when held.
 */
@Composable
private fun PostSection(card: Card, onOpenSource: (() -> Unit)?, onPictureHold: () -> Unit, pictureHoldLabel: String, onHold: (() -> Unit)?) {
    val colors = LocalCardSheetColors.current
    val brand = postBrand(card.type)
    val byline = remember(card) { postByline(card) }
    val previews = LocalVideoPreviews.current
    val video = remember(card) { readerVideo(card) }?.takeIf { previews != null }
    val words = remember(card) { postWords(card) }
    val title = card.reddit?.title?.trim()?.takeIf { card.type == "RedditPost" && it.isNotEmpty() }
    val second = listOfNotNull(byline?.handle, postDay(card)).joinToString(" · ").ifEmpty { null }
    val time = postTimeLine(card)
    val shape = RoundedCornerShape(24.dp)
    val body = TextStyle(color = brand.text, fontSize = 17.sp, lineHeight = 26.sp, fontFamily = MymindTheme.fonts.body)
    val muted = TextStyle(color = brand.muted, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body)
    Column(Modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(brand.paper)
        .padding(start = 18.dp, top = 18.dp, end = 18.dp, bottom = 16.dp)) {
        Column(Modifier.fillMaxWidth().held("Post actions", onHold)) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(byline?.name ?: sourceName(card), style = body.copy(fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.SemiBold),
                        maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (second != null) Text(second, style = muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                brand.icon?.let { Image(painterResource(it), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(brand.text)) }
            }
            if (title != null) Text(title, Modifier.padding(top = 12.dp), style = body.copy(fontFamily = MymindTheme.fonts.heading,
                fontWeight = FontWeight.SemiBold, fontSize = 22.sp, lineHeight = 28.sp))
            if (words.isNotEmpty()) Text(words, Modifier.padding(top = if (title != null) 10.dp else 12.dp), style = body)
        }
        val frame = Modifier.padding(top = 12.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp))
        val article = postArticle(card)
        if (article != null) PostArticleCard(card, article, brand, onOpenSource, onPictureHold, pictureHoldLabel)
        else if (video != null && previews != null) {
            CompositionLocalProvider(LocalCardSheetColors provides CardSheetColors(brand.paper, brand.paper, brand.text, colors.grabber,
                colors.scrim, brand.line, colors.shadow)) { CardVideo(card, video, previews, frame) }
        } else if (galleryPicture(card) != null) Box(frame) {
            CardPicture(card, play = card.post?.hasVideo == true, whole = true, modifier = Modifier
                .semantics { contentDescription = "Preview" }
                .combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, role = Role.Button,
                    onClickLabel = if (onOpenSource != null) "Open source" else null, onClick = { onOpenSource?.invoke() },
                    onLongClickLabel = pictureHoldLabel, onLongClick = onPictureHold,
                ))
        }
        if (time != null) Text(time, Modifier.padding(top = 12.dp), style = muted)
    }
}

/**
 * The X article a post carries, as X draws it in the post: a box ruled in the network's line colour holding the
 * article's cover whole, its title, its opening lines and Article on X. A tap opens the post on X, where the article
 * reads in full, and holding it calls [onHold], which says [holdLabel].
 */
@Composable
private fun PostArticleCard(card: Card, article: PostArticle, brand: PostBrand, onOpenSource: (() -> Unit)?, onHold: () -> Unit, holdLabel: String) {
    val shape = RoundedCornerShape(16.dp)
    val body = TextStyle(color = brand.text, fontSize = 15.sp, lineHeight = 22.sp, fontFamily = MymindTheme.fonts.body)
    val preview = article.preview?.trim()?.takeIf { it.isNotEmpty() }
    Column(Modifier.padding(top = 12.dp).fillMaxWidth().clip(shape).border(1.dp, brand.line, shape)
        .semantics { contentDescription = "Article, ${article.title.orEmpty()}" }
        .combinedClickable(
            interactionSource = null, indication = CardSheetIndication, role = Role.Button,
            onClickLabel = if (onOpenSource != null) "Open source" else null, onClick = { onOpenSource?.invoke() },
            onLongClickLabel = holdLabel, onLongClick = onHold,
        )) {
        if (galleryPicture(card) != null) CardPicture(card, play = false, whole = true)
        Column(Modifier.fillMaxWidth().padding(start = 12.dp, top = 10.dp, end = 12.dp, bottom = 12.dp)) {
            Text(article.title.orEmpty(), style = body.copy(fontSize = 17.sp, fontWeight = FontWeight.SemiBold))
            if (preview != null) Text(preview, Modifier.padding(top = 6.dp), style = body)
            Text("Article on X", Modifier.padding(top = 6.dp), style = body.copy(color = brand.muted, lineHeight = 21.sp))
        }
    }
}

/**
 * A Reddit post's top comments as they were when it was saved, each with its author over its words, ruled apart, as
 * people save a thread for its answers. It shows no points.
 */
@Composable
private fun CommentsSection(comments: List<RedditComment>, onHold: (() -> Unit)?) {
    val colors = LocalCardSheetColors.current
    ReaderSection(SECTION_PADDING, holdLabel = "Card actions", onHold = onHold) {
        SectionHeading(R.drawable.ic_comment, "Top comments", trailing = "When saved")
        comments.forEachIndexed { index, comment ->
            if (index > 0) Box(Modifier.padding(top = 12.dp).fillMaxWidth().height(1.dp).background(colors.active))
            Column(Modifier.padding(top = if (index > 0) 12.dp else 10.dp).semantics(mergeDescendants = true) {}) {
                Text("u/${comment.author.orEmpty().removePrefix("u/")}", style = detailStyle(colors).copy(fontFamily = MymindTheme.fonts.body))
                Text(comment.body.orEmpty(), Modifier.padding(top = 2.dp),
                    style = TextStyle(color = colors.text, fontSize = 16.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.body))
            }
        }
    }
}

/**
 * A PDF's head: its name, then what it is, how many pages it has and how long its words take to read once the phone
 * has read them, and Read in Reader Mode when it has words.
 */
@Composable
private fun PdfHeadSection(card: Card, pages: Int?, minutes: Int?, onRead: (() -> Unit)?) {
    val colors = LocalCardSheetColors.current
    ReaderSection(SECTION_PADDING) {
        Text(cardHeading(card), Modifier.semantics { heading() }, style = TextStyle(fontFamily = MymindTheme.fonts.heading,
            fontWeight = FontWeight.SemiBold, fontSize = 22.sp, lineHeight = 28.sp, color = colors.text))
        Text(pdfDetail(pages, minutes, savedLabel(card.created)), Modifier.padding(top = 4.dp), style = detailStyle(colors))
        if (onRead != null) {
            val shape = RoundedCornerShape(50)
            Row(Modifier.padding(top = 12.dp).heightIn(min = 44.dp).clip(shape).background(colors.active)
                    .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClick = onRead)
                    .padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Image(painterResource(R.drawable.readermodeicon), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.text))
                Text("Read in Reader Mode", style = TextStyle(color = colors.text, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = MymindTheme.fonts.body))
            }
        }
    }
}

/**
 * A PDF's pages one under another at the page's width, each on white with its number under it, drawn by the phone
 * as they come near the screen. Until the phone has the file, the first page shows as the server drew it.
 */
@Composable
private fun PdfPagesSection(card: Card, pdf: PdfPages?, onHold: () -> Unit) {
    if (pdf != null) {
        repeat(pdf.count) { index -> PdfPage(pdf, index, card.takeIf { index == 0 }, onHold) }
        return
    }
    if (galleryPicture(card) == null) return
    val ratio = card.cover?.let { cover -> if ((cover.width ?: 0) > 0 && (cover.height ?: 0) > 0) cover.width!!.toFloat() / cover.height!! else null }
    PdfPaper(ratio ?: PAGE_RATIO, onHold) { CardPicture(card, play = false, whole = true, modifier = Modifier.fillMaxSize()) }
}

/** One page of a PDF, drawn while it is within a couple of screens of the window and let go when it is further. */
@Composable
private fun PdfPage(pdf: PdfPages, index: Int, cover: Card?, onHold: () -> Unit) {
    val window = with(LocalDensity.current) { LocalConfiguration.current.screenHeightDp.dp.toPx() }
    var near by remember(pdf, index) { mutableStateOf(false) }
    var width by remember { mutableIntStateOf(0) }
    var drawn by remember(pdf, index) { mutableStateOf<ImageBitmap?>(null) }
    LaunchedEffect(pdf, index, near, width) {
        val target = width.coerceAtMost(PAGE_MAX_PX)
        drawn = if (near && target > 0) drawn?.takeIf { it.width == target } ?: pdf.draw(index, target) else null
    }
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        PdfPaper(pdf.ratio(index), onHold, Modifier
            .onSizeChanged { width = it.width }
            .onGloballyPositioned { coordinates ->
                val top = coordinates.positionInWindow().y
                near = top < window * PAGE_NEAR_SCREENS && top + coordinates.size.height > -window * PAGE_NEAR_SCREENS
            }
            .semantics { contentDescription = "Page ${index + 1} of ${pdf.count}" }) {
            val page = drawn
            if (page != null) Image(page, null, Modifier.fillMaxSize(), contentScale = ContentScale.FillBounds)
            else if (cover != null) CardPicture(cover, play = false, whole = true, modifier = Modifier.fillMaxSize())
        }
        Text("${index + 1} of ${pdf.count}", Modifier.clearAndSetSemantics {}, style = detailStyle(LocalCardSheetColors.current))
    }
}

/** A page's white paper at its own [ratio], with the reader's soft shadow; holding it opens the card's actions. */
@Composable
private fun PdfPaper(ratio: Float, onHold: () -> Unit, modifier: Modifier = Modifier, content: @Composable BoxScope.() -> Unit) {
    val shape = RoundedCornerShape(8.dp)
    Box(modifier.fillMaxWidth().aspectRatio(ratio.coerceIn(PAGE_RATIO_MIN, PAGE_RATIO_MAX)).cardSurfaceShadow(shape, note = true).clip(shape)
        .background(Color.White).held("Card actions", onHold), content = content)
}

private const val PAGE_RATIO = 0.707f
private const val PAGE_RATIO_MIN = 0.3f
private const val PAGE_RATIO_MAX = 3f
private const val PAGE_MAX_PX = 1440
private const val PAGE_NEAR_SCREENS = 2f

/**
 * Same Vibe for a product: the three cards nearest it in meaning, as Pinterest's "More to explore" shows pins like the
 * one open, each of which opens in this one's place.
 */
@Composable
private fun SameVibeSection(cards: List<Card>, onOpen: ((Card) -> Unit)?) {
    ReaderSection(SECTION_PADDING) {
        SectionHeading(R.drawable.samevibeicon, "Same Vibe", trailing = "From your mind")
        Spacer(Modifier.height(10.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            cards.take(SAME_VIBE_SHOWN).forEach { near ->
                val name = cardHeading(near)
                CardThumbnail(near, Modifier.weight(1f).height(104.dp).clip(RoundedCornerShape(12.dp))
                    .semantics { contentDescription = name }
                    .then(if (onOpen != null) Modifier.clickable(role = Role.Button, onClickLabel = "Open $name") { onOpen(near) } else Modifier))
            }
            repeat(SAME_VIBE_SHOWN - cards.size.coerceAtMost(SAME_VIBE_SHOWN)) { Spacer(Modifier.weight(1f)) }
        }
    }
}

private const val SAME_VIBE_SHOWN = 3

/** One part of the card's words under its small heading; the page text's heading also says how long it takes to read. */
@Composable
private fun TextSection(section: ReaderText, card: Card, onHold: (() -> Unit)? = null) {
    val colors = LocalCardSheetColors.current
    ReaderSection(SECTION_PADDING, holdLabel = "${section.part.heading} actions", onHold = onHold) {
        SectionHeading(partIcon(section.part), section.part.heading, trailing = if (section.part == ReaderPart.Page) readingTime(card) else null)
        Spacer(Modifier.height(8.dp))
        Text(section.text, style = TextStyle(color = colors.text, fontSize = 16.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.body))
    }
}

private val SECTION_PADDING = PaddingValues(start = 18.dp, top = 16.dp, end = 18.dp, bottom = 18.dp)

@DrawableRes
private fun partIcon(part: ReaderPart): Int = when (part) {
    ReaderPart.Post -> R.drawable.ic_quote
    ReaderPart.Note, ReaderPart.YourNote -> R.drawable.notes
    ReaderPart.Summary -> R.drawable.ic_summary
    ReaderPart.Page -> R.drawable.ic_page
    ReaderPart.Tags -> R.drawable.tags
    ReaderPart.Picture -> R.drawable.ic_image
}

/** A section's small heading: its icon and name, muted, with an optional detail or control at the end. */
@Composable
private fun SectionHeading(@DrawableRes icon: Int, label: String, trailing: String? = null, action: (@Composable () -> Unit)? = null) {
    val colors = LocalCardSheetColors.current
    val muted = colors.text.copy(alpha = 0.65f)
    Row(Modifier.fillMaxWidth().heightIn(min = 24.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Image(painterResource(icon), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(muted))
        Text(label, Modifier.weight(1f).semantics { heading() },
            style = TextStyle(color = muted, fontSize = 15.sp, lineHeight = 21.sp, fontWeight = FontWeight.SemiBold, fontFamily = MymindTheme.fonts.body))
        if (trailing != null) Text(trailing, style = detailStyle(colors))
        action?.invoke()
    }
}

/** Pully's muted 15/21 detail line. */
private fun detailStyle(colors: CardSheetColors) = TextStyle(color = colors.text.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp)

/**
 * The card's tags as quiet pills, with + Add when tags can be added. With [onRemove], after Edit tags from the tags'
 * held sheet, each shows a cross that takes it off and Done calls [onDone]. A card without tags shows the section only
 * when tags can be added.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagsSection(card: Card, onRemove: ((Tag) -> Unit)?, onDone: (() -> Unit)?, onAdd: (() -> Unit)?, onHold: (() -> Unit)? = null) {
    val tags = card.tags.orEmpty().filter { !it.name.isNullOrEmpty() }
    if (tags.isEmpty() && onAdd == null) return
    val colors = LocalCardSheetColors.current
    val haptic = LocalHapticFeedback.current
    val style = TextStyle(color = colors.text, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body)
    ReaderSection(SECTION_PADDING, holdLabel = "Tags actions", onHold = onHold) {
        SectionHeading(partIcon(ReaderPart.Tags), ReaderPart.Tags.heading, action = if (onRemove != null && onDone != null && tags.isNotEmpty()) ({
            Box(Modifier.heightIn(min = 32.dp).clip(RoundedCornerShape(50))
                .clickable(role = Role.Button) { onDone() }.padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
                Text("Done", style = style.copy(fontWeight = FontWeight.Medium))
            }
        }) else null)
        Spacer(Modifier.height(10.dp))
        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            tags.forEach { tag ->
                val removable = onRemove != null
                Row(
                    Modifier.heightIn(min = 32.dp).clip(RoundedCornerShape(50)).background(colors.active)
                        .padding(start = 12.dp, end = if (removable) 2.dp else 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(tag.name.orEmpty(), style = style)
                    if (removable) Image(
                        painterResource(R.drawable.ic_sheet_close),
                        contentDescription = "Remove tag ${tag.name.orEmpty()}",
                        colorFilter = ColorFilter.tint(style.color.copy(alpha = 0.65f)),
                        modifier = Modifier.size(32.dp).clip(CircleShape).clickable(role = Role.Button) {
                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                            onRemove?.invoke(tag)
                        }.padding(9.dp),
                    )
                }
            }
            if (onAdd != null) {
                val shape = RoundedCornerShape(50)
                Box(Modifier.heightIn(min = 32.dp).clip(shape).border(1.dp, colors.text.copy(alpha = 0.18f), shape)
                    .clickable(role = Role.Button, onClickLabel = "Add tags") { onAdd() }.padding(horizontal = 12.dp),
                    contentAlignment = Alignment.Center) { Text("+ Add", style = style) }
            }
        }
    }
}

/** Open source, in the ink of the page's one main action, and the card's actions, floating over a fade at the foot. */
@Composable
private fun ReaderFoot(card: Card, onOpenSource: (() -> Unit)?, onMenu: () -> Unit, modifier: Modifier = Modifier, openLabel: String? = null) {
    val colors = LocalCardSheetColors.current
    Box(modifier.fillMaxWidth()) {
        Box(Modifier.matchParentSize().background(Brush.verticalGradient(0f to colors.paper.copy(alpha = 0f), 0.6f to colors.paper)))
        Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 36.dp, bottom = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            if (onOpenSource != null) OpenSourceButton(sourceName(card), onOpenSource, Modifier.weight(1f), openLabel) else Spacer(Modifier.weight(1f))
            FootCircle(painterResource(R.drawable.ic_horizontal_menu), "Card actions", onMenu)
        }
    }
}

/** The 52dp ink pill that opens the original page, or with [label] such as "Open the PDF", the card's own file. */
@Composable
private fun OpenSourceButton(name: String, onClick: () -> Unit, modifier: Modifier = Modifier, label: String? = null) {
    val colors = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Row(
        modifier.height(52.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.text)
            .semantics { contentDescription = label ?: "Open source, $name" }
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 24.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label ?: "Open source", style = TextStyle(color = colors.paper, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium,
            fontFamily = MymindTheme.fonts.body), maxLines = 1)
        Image(painterResource(R.drawable.visitsourceicon), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.paper))
    }
}

/** A raised 52dp round control at the foot. */
@Composable
private fun FootCircle(icon: androidx.compose.ui.graphics.painter.Painter, label: String, onClick: () -> Unit) {
    val colors = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Box(
        Modifier.size(52.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.raised).semantics { contentDescription = label }
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button, onClickLabel = label) { haptic.tick(); onClick() },
        contentAlignment = Alignment.Center,
    ) { Image(icon, null, Modifier.size(24.dp), colorFilter = ColorFilter.tint(colors.text)) }
}
