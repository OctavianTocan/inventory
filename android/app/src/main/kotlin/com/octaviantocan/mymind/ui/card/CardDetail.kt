package com.octaviantocan.mymind.ui.card

import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.library.CardPicture
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.galleryPicture
import com.octaviantocan.mymind.ui.library.noteText
import com.octaviantocan.mymind.ui.library.postText
import com.octaviantocan.mymind.ui.library.proseText
import com.octaviantocan.mymind.ui.library.tileKind
import com.octaviantocan.mymind.ui.theme.MymindTheme

private val BODY_SIDE = 16.dp

/** A full-window card reader; contextual menu windows opened afterward remain above it. */
@Composable
fun CardDetailDialog(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    onRemoveTag: ((Tag) -> Unit)? = null,
    actionsProgress: Float = 0f,
    scrollState: ScrollState = rememberScrollState(),
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
            CardDetail(card, onClose, onMenu, modifier = Modifier.blur(8.dp * actionsProgress), onRemoveTag = onRemoveTag, scrollState = scrollState)
        }
    }
}

/**
 * A fixed, full-height reading page with one title and independently scrollable content. The card's type colour washes
 * the top edge, as it does on the card's sheets; the preview and the source link open the original.
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
) {
    val colors = if (MymindTheme.page.document.luminance() < 0.5f) DarkCardSheetColors else LightCardSheetColors
    val density = LocalDensity.current
    val uriHandler = LocalUriHandler.current
    val open = onOpenSource ?: { url: String -> runCatching { uriHandler.openUri(url) }; Unit }
    val source = card.url?.takeIf { it.startsWith("https://") || it.startsWith("http://") }?.let { url -> { open(url) } }
    val wash = cardAccent(card)?.copy(alpha = if (colors.paper.luminance() < 0.5f) 0.16f else 0.12f)
    BackHandler(onBack = onClose)
    CompositionLocalProvider(LocalCardSheetColors provides colors) {
        Column(modifier.fillMaxSize().background(colors.paper)
            .drawBehind {
                if (wash != null) drawRect(Brush.verticalGradient(listOf(wash, Color.Transparent), endY = 220.dp.toPx()))
            }
            .semantics {
                paneTitle = "Card details"
                dismiss { onClose(); true }
            }
            .windowInsetsPadding(WindowInsets.safeDrawing)) {
            Spacer(Modifier.height(16.dp))
            CardHeader(cardHeading(card), onClose = onClose, onMenu = onMenu, icons = headerIcons, titleInPill = true)
            val fade = with(density) { 20.dp.toPx() }
            Column(Modifier.fillMaxWidth().weight(1f)
                .sheetEdgeFade(color = colors.paper, topStrength = { (scrollState.value / fade).coerceIn(0f, 1f) })
                .verticalScroll(scrollState)) {
                CardBody(card, onRemoveTag, source, onMenu)
            }
        }
    }
}

/**
 * @param card A card.
 * @return The reader's paragraphs, in order: the post or note itself, the saved description, the captured page text
 *   and the user's own note, without blanks or repeats.
 */
internal fun readerParagraphs(card: Card): List<String> {
    val prose = card.prose?.let(::proseText)
    val parts = when (tileKind(card)) {
        TileKind.Note -> listOf(noteText(card), card.note)
        TileKind.Post -> listOf(postText(card), prose, card.note)
        else -> listOf(card.description, prose, card.note)
    }
    return parts.mapNotNull { it?.trim()?.takeIf(String::isNotEmpty) }.distinct()
}

/** The card's own content: picture, source, text and tags. The detail header supplies the title. */
@Composable
private fun CardBody(card: Card, onRemoveTag: ((Tag) -> Unit)?, onOpenSource: (() -> Unit)?, onMenu: () -> Unit) {
    val body = TextStyle(color = LocalCardSheetColors.current.text, fontSize = 19.sp, lineHeight = 1.5.em, fontFamily = MymindTheme.fonts.body)
    Column(Modifier.fillMaxWidth().padding(horizontal = BODY_SIDE).padding(top = 20.dp, bottom = 30.dp)) {
        val kind = tileKind(card)
        if (galleryPicture(card) != null) {
            CardPicture(card, play = kind == TileKind.Video, modifier = Modifier
                .semantics { contentDescription = "Preview" }
                .combinedClickable(
                    interactionSource = null, indication = CardSheetIndication, role = Role.Button,
                    onClickLabel = if (onOpenSource != null) "Open source" else null, onClick = { onOpenSource?.invoke() },
                    onLongClickLabel = "Card actions", onLongClick = onMenu,
                ))
            Spacer(Modifier.height(16.dp))
        }
        if (onOpenSource != null) {
            SourceLink(sourceName(card), onOpenSource)
            Spacer(Modifier.height(24.dp))
        }
        readerParagraphs(card).forEachIndexed { index, text ->
            if (index > 0) Spacer(Modifier.height(18.dp))
            Text(text, style = body)
        }
        val tags = card.tags.orEmpty().filter { !it.name.isNullOrEmpty() }
        if (tags.isNotEmpty()) {
            Spacer(Modifier.height(28.dp))
            TagChips(tags, onRemoveTag)
        }
    }
}

/** Where the card came from, for the source link: its site, else its domain, else the link's host. */
private fun sourceName(card: Card): String = card.siteName?.takeIf { it.isNotBlank() }
    ?: card.domain?.takeIf { it.isNotBlank() }
    ?: card.url?.let { runCatching { java.net.URI(it).host }.getOrNull() }?.removePrefix("www.")
    ?: "Source"

/** A raised pill that opens the original page. */
@Composable
private fun SourceLink(name: String, onClick: () -> Unit) {
    val colors = LocalCardSheetColors.current
    val shape = RoundedCornerShape(50)
    val haptic = rememberSheetHaptics()
    Row(
        Modifier.heightIn(min = 40.dp).cardSurfaceShadow(shape).clip(shape).background(colors.raised)
            .semantics { contentDescription = "Open source, $name" }
            .clickable(remember { MutableInteractionSource() }, CardSheetIndication, role = Role.Button) { haptic.tick(); onClick() }
            .padding(horizontal = 14.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(painterResource(R.drawable.visitsourceicon), null, Modifier.size(18.dp), colorFilter = ColorFilter.tint(colors.text))
        Text(name, style = TextStyle(color = colors.text, fontSize = 15.sp, lineHeight = 20.sp, fontFamily = MymindTheme.fonts.body),
            maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** The card's tags as quiet pills; Edit shows a cross on each, which takes that tag off. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagChips(tags: List<Tag>, onRemove: ((Tag) -> Unit)?) {
    val colors = LocalCardSheetColors.current
    val haptic = LocalHapticFeedback.current
    var editing by remember { mutableStateOf(false) }
    val style = TextStyle(color = colors.text.copy(alpha = 0.72f), fontSize = 14.sp, lineHeight = 20.sp, fontFamily = MymindTheme.fonts.body)
    FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
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
                    colorFilter = ColorFilter.tint(style.color),
                    modifier = Modifier.size(32.dp).clip(CircleShape).clickable(role = Role.Button) {
                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                        onRemove?.invoke(tag)
                    }.padding(9.dp),
                )
            }
        }
        if (onRemove != null) {
            Box(
                Modifier.heightIn(min = 32.dp).clip(RoundedCornerShape(50))
                    .clickable(role = Role.Button) { editing = !editing }.padding(horizontal = 12.dp),
                contentAlignment = Alignment.Center,
            ) { Text(if (editing) "Done" else "Edit tags", style = style.copy(color = colors.text, fontWeight = FontWeight.Medium)) }
        }
    }
}
