package com.octaviantocan.mymind.ui.card

import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.library.CardPicture
import com.octaviantocan.mymind.ui.library.TileKind
import com.octaviantocan.mymind.ui.library.cardHeading
import com.octaviantocan.mymind.ui.library.noteText
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

/** A fixed, full-height reading page with one title and independently scrollable content. */
@Composable
internal fun CardDetail(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    modifier: Modifier = Modifier,
    scrollState: ScrollState = rememberScrollState(),
    headerIcons: CardHeaderIcons = defaultCardHeaderIcons(),
    onRemoveTag: ((Tag) -> Unit)? = null,
) {
    val colors = if (MymindTheme.page.document.luminance() < 0.5f) DarkCardSheetColors else LightCardSheetColors
    val density = LocalDensity.current
    BackHandler(onBack = onClose)
    CompositionLocalProvider(LocalCardSheetColors provides colors) {
        Column(modifier.fillMaxSize().background(colors.paper)
            .semantics {
                paneTitle = "Card details"
                dismiss { onClose(); true }
            }
            .windowInsetsPadding(WindowInsets.safeDrawing)) {
            Spacer(Modifier.height(16.dp))
            CardHeader(cardHeading(card), onClose = onClose, onMenu = onMenu, icons = headerIcons)
            val fade = with(density) { 20.dp.toPx() }
            Column(Modifier.fillMaxWidth().weight(1f)
                .sheetEdgeFade(color = colors.paper, topStrength = { (scrollState.value / fade).coerceIn(0f, 1f) })
                .verticalScroll(scrollState)) {
                CardBody(card, onRemoveTag)
            }
        }
    }
}

/** The card's tags as chips; tapping a chip's cross takes the tag off. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagChips(tags: List<Tag>, style: TextStyle, onRemove: (Tag) -> Unit) {
    val haptic = LocalHapticFeedback.current
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        tags.forEach { tag ->
            Row(
                Modifier
                    .clip(RoundedCornerShape(50))
                    .background(MymindTheme.page.surface)
                    .padding(start = 14.dp, end = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(tag.name.orEmpty(), Modifier.padding(vertical = 6.dp), style = style)
                Text(
                    "\u00D7",
                    Modifier
                        .semantics { contentDescription = "Remove tag ${tag.name.orEmpty()}" }
                        .clickable(role = Role.Button) {
                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                            onRemove(tag)
                        }
                        .padding(horizontal = 8.dp, vertical = 6.dp),
                    style = style,
                )
            }
        }
    }
}

/** The card's own content: picture, text and tags. The detail header supplies the title. */
@Composable
private fun CardBody(card: Card, onRemoveTag: ((Tag) -> Unit)?) {
    val page = MymindTheme.page
    val fonts = MymindTheme.fonts
    val body = TextStyle(color = LocalCardSheetColors.current.text, fontSize = 19.sp, lineHeight = 1.5.em, fontFamily = FontFamily.Serif)
    Column(Modifier.fillMaxWidth().padding(horizontal = BODY_SIDE, vertical = 30.dp)) {
        val kind = tileKind(card)
        if (kind == TileKind.Media || kind == TileKind.Video) {
            CardPicture(card, play = kind == TileKind.Video)
            Spacer(Modifier.height(24.dp))
        }
        val text = if (kind == TileKind.Note) noteText(card) else listOfNotNull(card.description, card.note).joinToString("\n\n")
        if (text.isNotBlank()) Text(text, style = body)
        val tags = card.tags.orEmpty().filter { !it.name.isNullOrEmpty() }
        if (tags.isNotEmpty()) {
            Spacer(Modifier.height(24.dp))
            val style = TextStyle(color = page.muted, fontSize = 15.sp, fontFamily = fonts.nunitoSemiBold)
            if (onRemoveTag == null) {
                Text(tags.joinToString("  ") { it.name.orEmpty() }, style = style)
            } else {
                TagChips(tags, style, onRemoveTag)
            }
        }
    }
}
