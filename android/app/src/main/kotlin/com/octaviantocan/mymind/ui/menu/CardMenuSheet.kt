package com.octaviantocan.mymind.ui.menu

import android.view.WindowManager
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedContentScope
import androidx.compose.animation.EnterExitState
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.LocalIndication
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
import androidx.compose.ui.draw.BlurredEdgeTreatment
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.*
import androidx.compose.ui.semantics.*
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import com.octaviantocan.mymind.ui.card.*
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** A new sheet instance owns its drag position, reveal and native landing feedback. */
@Composable
internal fun rememberCardMenuPosition(): CardSheetPosition {
    val haptic = rememberSheetHaptics()
    return remember { CardSheetPosition(haptic::land, haptic::tick) }
}

/** The card's auxiliary pages in Pully's own modal presentation, with no Material sheet underneath. */
@Composable
internal fun CardMenuSheet(
    items: List<CardMenuItem>,
    onSelect: (CardMenuItem) -> Unit,
    onDismiss: () -> Unit,
    position: CardSheetPosition = rememberCardMenuPosition(),
    icons: CardMenuIcons = defaultCardMenuIcons(),
    submenu: CardSubmenu? = null,
    onBack: () -> Unit = {},
    submenuContent: @Composable (CardSubmenu) -> Unit = {},
    accent: androidx.compose.ui.graphics.Color? = null,
) {
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val latestDismiss by rememberUpdatedState(onDismiss)
    val scrolls = remember { mutableMapOf<CardSubmenu?, ScrollState>() }
    PullySheetHost(
        onDismissRequest = { if (submenu != null) onBack() else position.dismiss(scope, motion) { latestDismiss() } },
        accent = accent,
    ) {
        val colors = cardMenuSheetColors(accent, MymindTheme.colors.dark)
        CardMenuPanel(submenu?.title ?: "Card actions", onDismiss, position = position, accent = accent,
            onBack = if (submenu != null) onBack else null, scrollState = null,
            titleContent = {
                CardMenuPageSwitch(submenu, Modifier.fillMaxWidth(), centered = true) { page ->
                    Text(page?.title ?: "Card actions", Modifier.fillMaxWidth())
                }
            }) {
            CardMenuPageSwitch(submenu, Modifier.fillMaxWidth(), slow = true) { page ->
                val scroll = scrolls.getOrPut(page) { ScrollState(0) }
                val fade = with(LocalDensity.current) { 20.dp.toPx() }
                Column(Modifier.fillMaxWidth()
                    .sheetEdgeFade(color = colors.paper, topStrength = { (scroll.value / fade).coerceIn(0f, 1f) })
                    .verticalScroll(scroll).navigationBarsPadding()) {
                    if (page == null) CardMenuContent(items, onSelect, icons = icons, enabled = !position.closing)
                    else submenuContent(page)
                }
            }
        }
    }
}

/** The same Pully sheet window used by card pages and the changelog. */
@Composable
private fun PullySheetHost(onDismissRequest: () -> Unit, accent: androidx.compose.ui.graphics.Color? = null, content: @Composable () -> Unit) {
    val colors = cardMenuSheetColors(accent, MymindTheme.colors.dark)
    Dialog(
        onDismissRequest = onDismissRequest,
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val view = LocalView.current
        SideEffect {
            val window = (view.parent as? DialogWindowProvider)?.window ?: return@SideEffect
            window.setDimAmount(0f)
            window.setWindowAnimations(0)
            window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)
            window.isNavigationBarContrastEnforced = false
            WindowCompat.getInsetsController(window, view).isAppearanceLightNavigationBars = colors.paper.luminance() > 0.5f
        }
        content()
    }
}

/** Pully's plain-title, scrolling bottom sheet, with its shared drag, Back and dismiss behavior. */
@Composable
internal fun PullySheet(title: String, onClose: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    val position = rememberCardMenuPosition()
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val latestClose by rememberUpdatedState(onClose)
    PullySheetHost(onDismissRequest = { position.dismiss(scope, motion) { latestClose() } }) {
        CardMenuPanel(title, onClose, position = position, closeLabel = "Close") {
            Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 30.dp), content = content)
        }
    }
}

/** Pully's bottom-attached paper, shared by the live dialog and native screenshot fixtures. */
@Composable
internal fun CardMenuPanel(
    title: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    position: CardSheetPosition = rememberCardMenuPosition(),
    accent: androidx.compose.ui.graphics.Color? = null,
    onBack: (() -> Unit)? = null,
    scrollState: ScrollState? = rememberScrollState(),
    titleContent: (@Composable () -> Unit)? = null,
    closeLabel: String = "Close card actions",
    content: @Composable ColumnScope.() -> Unit,
) {
    val colors = cardMenuSheetColors(accent, MymindTheme.colors.dark)
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    val motion = rememberSheetMotionEnabled()
    val haptic = rememberSheetHaptics()
    val latestClose by rememberUpdatedState(onDismiss)
    val dismiss = { position.dismiss(scope, motion) { latestClose() } }
    val latestDismiss by rememberUpdatedState(dismiss)
    val latestMotion by rememberUpdatedState(motion)
    val top = with(density) { WindowInsets.statusBars.getTop(this).toDp() }
    val screen = LocalConfiguration.current.screenHeightDp.dp
    val tall = (screen - maxOf(top, 16.dp) - 44.dp).coerceAtLeast(1.dp)
    val window = LocalWindowInfo.current.containerSize.height
    position.window = if (window > 0) window.toFloat() else with(density) { screen.toPx() }
    position.updateHalf(position.window / 2f)
    LaunchedEffect(position.half) { position.follow(scope, motion) }
    val keyboard = WindowInsets.ime.getBottom(density) > 0
    val latestKeyboard by rememberUpdatedState(keyboard)
    val focus = LocalFocusManager.current
    LaunchedEffect(keyboard) { if (keyboard) position.lift(scope, latestMotion) }
    LaunchedEffect(position.tall) { if (!position.tall && latestKeyboard) focus.clearFocus() }
    val speed = with(density) { 200.dp.toPx() }
    val connection = remember(position, scope, speed) { CardSheetScroll(position, scope, speed, { latestMotion }, { latestDismiss() }) }
    val drag = rememberDraggableState { position.dragBy(it) }
    BackHandler { onBack?.invoke() ?: dismiss() }
    CompositionLocalProvider(LocalCardMenuAccent provides accent, LocalCardSheetColors provides colors,
        LocalIndication provides CardSheetIndication) {
        Box(modifier.fillMaxSize()) {
            Box(Modifier.fillMaxSize().graphicsLayer { alpha = position.scrim }.background(colors.scrim)
                .clickable(remember { MutableInteractionSource() }, indication = null, role = Role.Button,
                    onClickLabel = closeLabel, onClick = { haptic.tick(); dismiss() }))
            Box(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(tall)
                    .onSizeChanged { position.measured(it.height, scope, motion) }
                    .graphicsLayer {
                        translationY = position.shift(size.height)
                        alpha = position.reveal
                        val radius = 2.dp.toPx() * (1f - position.reveal)
                        renderEffect = if (radius > 0f) BlurEffect(radius, radius, TileMode.Decal) else null
                    }
                    .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp)).background(colors.paper)
                    .semantics {
                        paneTitle = title
                        dismiss { dismiss(); true }
                        expand { position.expand(scope, true, latestMotion); true }
                        collapse { position.expand(scope, false, latestMotion); true }
                    }
                    .nestedScroll(connection)
                    .draggable(drag, Orientation.Vertical, enabled = !position.closing,
                        onDragStopped = { position.settle(scope, it, speed, latestMotion, latestDismiss) }),
            ) {
                if (accent != null) {
                    Box(Modifier.fillMaxWidth().height(64.dp).background(
                        androidx.compose.ui.graphics.Brush.verticalGradient(
                            listOf(accent.copy(alpha = if (MymindTheme.colors.dark) 0.14f else 0.10f), androidx.compose.ui.graphics.Color.Transparent),
                        ),
                    ))
                }
                Column(Modifier.fillMaxWidth().heightIn(max = tall).imePadding()
                    .onSizeChanged { position.measuredContent(it.height, scope, motion) }) {
                    Box(Modifier.padding(top = 10.dp, bottom = 14.dp).align(Alignment.CenterHorizontally)
                        .size(36.dp, 4.dp).clip(CircleShape).background(colors.grabber))
                    CardHeader(title, onClose = dismiss, enabled = !position.closing, closeLabel = closeLabel,
                        onBack = onBack, titleContent = titleContent)
                    val fade = with(density) { 20.dp.toPx() }
                    Column(Modifier.fillMaxWidth().weight(1f, fill = false).then(
                        if (scrollState != null) Modifier
                            .sheetEdgeFade(color = colors.paper, topStrength = { (scrollState.value / fade).coerceIn(0f, 1f) })
                            .verticalScroll(scrollState).navigationBarsPadding()
                        else Modifier,
                    ), content = content)
                }
            }
        }
    }
}

/** Pully's BlurSwitch, scoped to this sheet's root and child pages (DESIGN rules 78, 105 and 115). */
@Composable
private fun CardMenuPageSwitch(
    page: CardSubmenu?,
    modifier: Modifier = Modifier,
    centered: Boolean = false,
    slow: Boolean = false,
    content: @Composable AnimatedContentScope.(CardSubmenu?) -> Unit,
) {
    val moving = rememberSheetMotionEnabled()
    val step = with(LocalDensity.current) { 8.dp.roundToPx() }
    val inMs = if (slow) CardSheetMotion.MEDIUM else CardSheetMotion.FAST
    val outMs = if (slow) CardSheetMotion.FAST else 150
    AnimatedContent(
        targetState = page,
        modifier = modifier,
        contentAlignment = if (centered) Alignment.Center else Alignment.TopStart,
        transitionSpec = {
            if (!moving) EnterTransition.None togetherWith ExitTransition.None
            else {
                val direction = if ((targetState?.depth ?: 0) < (initialState?.depth ?: 0)) -1 else 1
                val travel = { width: Int -> if (slow) (width * 0.34f).toInt() else step }
                ((slideInHorizontally(tween(inMs, easing = CardSheetMotion.SmoothOut)) { direction * travel(it) }
                    + fadeIn(tween(inMs, easing = CardSheetMotion.SmoothOut))) togetherWith
                    (slideOutHorizontally(tween(outMs, easing = CardSheetMotion.SmoothOut)) { -direction * travel(it) }
                    + fadeOut(tween(outMs, easing = CardSheetMotion.SmoothOut))))
                    .using(SizeTransform(clip = true) { _, _ -> tween(inMs, easing = CardSheetMotion.SmoothOut) })
            }
        },
        label = "cardMenuPage",
    ) { target ->
        val blur by transition.animateDp(
            transitionSpec = { tween(if (moving) inMs else 0, easing = CardSheetMotion.SmoothOut) }, label = "cardMenuPageBlur",
        ) { if (it == EnterExitState.Visible) 0.dp else 3.dp }
        Box(if (moving && blur > 0.dp) Modifier.blur(blur, BlurredEdgeTreatment.Unbounded) else Modifier) { content(target) }
    }
}
