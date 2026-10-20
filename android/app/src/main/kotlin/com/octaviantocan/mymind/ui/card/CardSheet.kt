package com.octaviantocan.mymind.ui.card

import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.semantics.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
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
import com.octaviantocan.mymind.ui.menu.DarkMenuColors

private val BODY_SIDE = 16.dp

/** Pully's edge-to-edge modal window; card menus opened afterward remain above it. */
@Composable
fun CardSheetDialog(card: Card, onClose: () -> Unit, onMenu: () -> Unit, onRemoveTag: ((Tag) -> Unit)? = null, actionsVisible: Boolean = false) {
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val haptic = rememberSheetHaptics()
    val position = remember { CardSheetPosition(haptic::land, haptic::tick) }
    val latestClose by rememberUpdatedState(onClose)
    val actionsProgress by animateFloatAsState(if (actionsVisible) 1f else 0f, label = "cardActions")
    Dialog(
        onDismissRequest = { position.dismiss(scope, motion) { latestClose() } },
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
            WindowCompat.getInsetsController(window, view).isAppearanceLightNavigationBars = light
        }
        Box(Modifier.fillMaxSize()) {
            CardSheet(card, onClose, onMenu, modifier = Modifier.blur(8.dp * actionsProgress), onRemoveTag = onRemoveTag, position = position)
            if (actionsProgress > 0f) Box(Modifier.fillMaxSize().background(DarkMenuColors.scrim.copy(alpha = DarkMenuColors.scrim.alpha * actionsProgress)))
        }
    }
}

/** Pully's sheet panel: content-sized up to half the window, a fixed header, then a scrollable page. */
@Composable
internal fun CardSheet(
    card: Card,
    onClose: () -> Unit,
    onMenu: () -> Unit,
    modifier: Modifier = Modifier,
    scrollState: ScrollState = rememberScrollState(),
    headerIcons: CardHeaderIcons = defaultCardHeaderIcons(),
    onRemoveTag: ((Tag) -> Unit)? = null,
    position: CardSheetPosition? = null,
) {
    val colors = if (MymindTheme.page.document.luminance() < 0.5f) DarkCardSheetColors else LightCardSheetColors
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val haptic = rememberSheetHaptics()
    val sheet = position ?: remember { CardSheetPosition(haptic::land, haptic::tick) }
    val latestClose by rememberUpdatedState(onClose)
    val requestDismiss = { sheet.dismiss(scope, motion) { latestClose() } }
    val latestDismiss by rememberUpdatedState(requestDismiss)
    val latestMotion by rememberUpdatedState(motion)
    val speed = with(density) { 200.dp.toPx() }
    val connection = remember(sheet, scope, speed) { CardSheetScroll(sheet, scope, speed, { latestMotion }, { latestDismiss() }) }
    val drag = rememberDraggableState { sheet.dragBy(it) }
    val top = with(density) { WindowInsets.statusBars.getTop(this).toDp() }
    val screen = LocalConfiguration.current.screenHeightDp.dp
    val tall = (screen - maxOf(top, 16.dp) - 44.dp).coerceAtLeast(1.dp)
    val window = LocalWindowInfo.current.containerSize.height
    sheet.window = if (window > 0) window.toFloat() else with(density) { screen.toPx() }
    sheet.updateHalf(sheet.window / 2f)
    LaunchedEffect(sheet.half) { sheet.follow(scope, motion) }
    val keyboard = WindowInsets.ime.getBottom(density) > 0
    val latestKeyboard by rememberUpdatedState(keyboard)
    val focus = LocalFocusManager.current
    LaunchedEffect(keyboard) { if (keyboard) sheet.lift(scope, latestMotion) }
    LaunchedEffect(sheet.tall) { if (!sheet.tall && latestKeyboard) focus.clearFocus() }
    BackHandler { requestDismiss() }
    CompositionLocalProvider(LocalCardSheetColors provides colors) {
        Box(modifier.fillMaxSize()) {
            Box(Modifier.fillMaxSize().graphicsLayer { alpha = sheet.scrim }.background(colors.scrim)
                .clickable(remember { MutableInteractionSource() }, indication = null, role = Role.Button, onClickLabel = "Close card", onClick = { haptic.tick(); requestDismiss() }))
            Box(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(tall)
                    .onSizeChanged { sheet.measured(it.height, scope, motion) }
                    .graphicsLayer {
                        translationY = sheet.shift(size.height)
                        alpha = sheet.reveal
                        val radius = 2.dp.toPx() * (1f - sheet.reveal)
                        renderEffect = if (radius > 0f) BlurEffect(radius, radius, TileMode.Decal) else null
                    }
                    .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp)).background(colors.paper)
                    .semantics {
                        paneTitle = "Card details"
                        dismiss { requestDismiss(); true }
                        expand { sheet.expand(scope, true, latestMotion); true }
                        collapse { sheet.expand(scope, false, latestMotion); true }
                    }
                    .nestedScroll(connection)
                    .draggable(drag, Orientation.Vertical, enabled = !sheet.closing, onDragStopped = { velocity -> sheet.settle(scope, velocity, speed, latestMotion, requestDismiss) }),
            ) {
                Column(Modifier.fillMaxWidth().heightIn(max = tall).imePadding().onSizeChanged { sheet.measuredContent(it.height, scope, motion) }) {
                    Box(Modifier.padding(top = 10.dp, bottom = 20.dp).align(Alignment.CenterHorizontally).size(36.dp, 4.dp).clip(CircleShape).background(colors.grabber))
                    CardHeader(cardHeading(card), onClose = requestDismiss, onMenu = onMenu, icons = headerIcons, enabled = !sheet.closing)
                    val fade = with(density) { 20.dp.toPx() }
                    Column(Modifier.fillMaxWidth().weight(1f, fill = false)
                        .sheetEdgeFade(color = colors.paper, topStrength = { (scrollState.value / fade).coerceIn(0f, 1f) })
                        .verticalScroll(scrollState).navigationBarsPadding()) {
                        CardBody(card, onRemoveTag)
                    }
                }
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

/** The card's own content: picture, centred title, text and tags. */
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
        if (kind != TileKind.Note) {
            Text(
                cardHeading(card),
                Modifier.fillMaxWidth(),
                style = TextStyle(color = LocalCardSheetColors.current.text, fontSize = 31.sp, lineHeight = 1.06.em, fontFamily = fonts.louize, textAlign = TextAlign.Center),
            )
            Spacer(Modifier.height(30.dp))
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
