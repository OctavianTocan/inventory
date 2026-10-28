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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
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
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.CardPicture
import com.octaviantocan.mymind.ui.library.LocalVideoPreviews
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.readerVideo
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
)

/**
 * What the reader shows about a card that the card itself does not hold.
 *
 * @property spaces Names of the spaces the card is in, in the user's order.
 * @property recent Cards opened before this one, latest first.
 */
@Immutable
data class ReaderDetails(val spaces: List<String> = emptyList(), val recent: List<Card> = emptyList())

/**
 * A full-window card reader; contextual menu windows opened afterward remain above it.
 *
 * @param actionsProgress How far the card's sheet has risen over it, from 0 to 1; the reader blurs with it.
 * @param shrink How far Delete card's hold has come, from 0 to 1; the reader shrinks and fades with it.
 */
@Composable
fun CardDetailDialog(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    onRemoveTag: ((Tag) -> Unit)? = null,
    actionsProgress: Float = 0f,
    shrink: Float = 0f,
    scrollState: ScrollState = rememberScrollState(),
    actions: ReaderActions = ReaderActions(),
    details: ReaderDetails = ReaderDetails(),
) {
    Dialog(
        onDismissRequest = onClose,
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val view = LocalView.current
        val light = MymindTheme.page.document.luminance() > 0.5f
        SideEffect {
            val window = (view.parent as? DialogWindowProvider)?.window ?: return@SideEffect
            window.setDimAmount(0f)
            window.setWindowAnimations(0)
            window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
            window.isNavigationBarContrastEnforced = false
            WindowCompat.getInsetsController(window, view).apply {
                isAppearanceLightStatusBars = light
                isAppearanceLightNavigationBars = light
            }
        }
        Box(Modifier.fillMaxSize()) {
            CardDetail(card, onClose, onMenu, modifier = Modifier.blur(8.dp * actionsProgress).shrunk { shrink }, onRemoveTag = onRemoveTag,
                scrollState = scrollState, actions = actions, details = details)
        }
    }
}

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
 * The card's type colour washes the top edge, as it does on the card's sheets.
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
) {
    val colors = if (MymindTheme.page.document.luminance() < 0.5f) DarkCardSheetColors else LightCardSheetColors
    val density = LocalDensity.current
    val uriHandler = LocalUriHandler.current
    val open = onOpenSource ?: { url: String -> runCatching { uriHandler.openUri(url) }; Unit }
    val source = sourceUrl(card)?.let { url -> { open(url) } }
    val wash = cardAccent(card)?.copy(alpha = if (colors.paper.luminance() < 0.5f) 0.16f else 0.12f)
    var named by rememberSaveable(card.id) { mutableStateOf(false) }
    BackHandler(onBack = onClose)
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
                ReaderHeader(cardHeading(card), named, onBack = onClose, onName = { named = !named }, onShare = actions.onShare, icons = headerIcons)
                Box(Modifier.fillMaxWidth().weight(1f)) {
                    val fade = with(density) { 20.dp.toPx() }
                    Column(Modifier.fillMaxSize()
                        .sheetEdgeFade(color = colors.paper, topStrength = { (scrollState.value / fade).coerceIn(0f, 1f) })
                        .verticalScroll(scrollState)) {
                        ReaderBody(card, onRemoveTag, actions.onAddTags, source, onMenu)
                    }
                    ReaderFoot(card, source, onMenu, Modifier.align(Alignment.BottomCenter))
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
) {
    Row(modifier.fillMaxWidth().zIndex(1f).padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        // The reader returns to the gallery, so it leads with Back rather than a closing cross.
        CardHeaderButton(icons.back ?: painterResource(R.drawable.ic_sheet_back), "Back", onBack, true)
        NamePill(title, open, onName, Modifier.weight(1f))
        if (onShare != null) CardHeaderButton(icons.share ?: painterResource(R.drawable.ic_share), "Share", onShare, true)
        else Spacer(Modifier.size(44.dp))
    }
}

/** The card's name on one line with a caret, in a raised pill; tapping it shows or hides the name's card. */
@Composable
private fun NamePill(title: String, open: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
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

/** The card's parts, each in its own raised section: picture and title, its words, tags, the user's note and the page. */
@Composable
private fun ReaderBody(card: Card, onRemoveTag: ((Tag) -> Unit)?, onAddTags: (() -> Unit)?, onOpenSource: (() -> Unit)?, onMenu: () -> Unit) {
    val sections = readerSections(card)
    Column(Modifier.fillMaxWidth().padding(horizontal = BODY_SIDE).padding(top = 12.dp, bottom = FOOT_ROOM),
        verticalArrangement = Arrangement.spacedBy(12.dp)) {
        HeadSection(card, onOpenSource, onMenu)
        sections.filter { it.part in LEAD_PARTS }.forEach { TextSection(it, card) }
        TagsSection(card, onRemoveTag, onAddTags)
        sections.filter { it.part !in LEAD_PARTS }.forEach { TextSection(it, card) }
    }
}

/** A raised reading section with Pully's note shadow and 24dp corners. */
@Composable
private fun ReaderSection(padding: PaddingValues, modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val shape = RoundedCornerShape(24.dp)
    Column(modifier.fillMaxWidth().cardSurfaceShadow(shape, note = true).clip(shape).background(LocalCardSheetColors.current.raised).padding(padding),
        content = content)
}

/**
 * The picture, or the video the server keeps, over the title, who made the card and when it was saved. A tap on the
 * picture opens the source and holding it opens the card's actions. A card with neither picture nor title has none.
 */
@Composable
private fun HeadSection(card: Card, onOpenSource: (() -> Unit)?, onMenu: () -> Unit) {
    val colors = LocalCardSheetColors.current
    val previews = LocalVideoPreviews.current
    val video = remember(card) { readerVideo(card) }?.takeIf { previews != null }
    val picture = galleryPicture(card) != null
    val title = card.title?.trim()?.takeIf { it.isNotEmpty() }
    val lines = listOfNotNull(bylineOf(card), listOfNotNull(savedLabel(card.created), readingTime(card)).joinToString(" · ").ifEmpty { null })
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
                    onLongClickLabel = "Card actions", onLongClick = onMenu,
                ))
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

/** One part of the card's words under its small heading; the page text's heading also says how long it takes to read. */
@Composable
private fun TextSection(section: ReaderText, card: Card) {
    val colors = LocalCardSheetColors.current
    ReaderSection(SECTION_PADDING) {
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
 * The card's tags as quiet pills, with + Add when tags can be added. Edit tags shows a cross on each, which takes that
 * tag off, and Done hides them again. A card without tags shows the section only when tags can be added.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagsSection(card: Card, onRemove: ((Tag) -> Unit)?, onAdd: (() -> Unit)?) {
    val tags = card.tags.orEmpty().filter { !it.name.isNullOrEmpty() }
    if (tags.isEmpty() && onAdd == null) return
    val colors = LocalCardSheetColors.current
    val haptic = LocalHapticFeedback.current
    var editing by remember(card.id) { mutableStateOf(false) }
    val style = TextStyle(color = colors.text, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body)
    ReaderSection(SECTION_PADDING) {
        SectionHeading(R.drawable.tags, "Tags", action = if (onRemove != null && tags.isNotEmpty()) ({
            Box(Modifier.heightIn(min = 32.dp).clip(RoundedCornerShape(50))
                .clickable(role = Role.Button) { editing = !editing }.padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
                Text(if (editing) "Done" else "Edit tags", style = style.copy(fontWeight = FontWeight.Medium))
            }
        }) else null)
        Spacer(Modifier.height(10.dp))
        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            tags.forEach { tag ->
                val removable = editing && onRemove != null
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
private fun ReaderFoot(card: Card, onOpenSource: (() -> Unit)?, onMenu: () -> Unit, modifier: Modifier = Modifier) {
    val colors = LocalCardSheetColors.current
    Box(modifier.fillMaxWidth()) {
        Box(Modifier.matchParentSize().background(Brush.verticalGradient(0f to colors.paper.copy(alpha = 0f), 0.6f to colors.paper)))
        Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 36.dp, bottom = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            if (onOpenSource != null) OpenSourceButton(sourceName(card), onOpenSource, Modifier.weight(1f)) else Spacer(Modifier.weight(1f))
            FootCircle(painterResource(R.drawable.ic_horizontal_menu), "Card actions", onMenu)
        }
    }
}

/** The 52dp ink pill that opens the original page. */
@Composable
private fun OpenSourceButton(name: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val colors = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Row(
        modifier.height(52.dp).cardSurfaceShadow(CircleShape).clip(CircleShape).background(colors.text)
            .semantics { contentDescription = "Open source, $name" }
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 24.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text("Open source", style = TextStyle(color = colors.paper, fontSize = 17.sp, lineHeight = 22.sp, fontWeight = FontWeight.Medium,
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
