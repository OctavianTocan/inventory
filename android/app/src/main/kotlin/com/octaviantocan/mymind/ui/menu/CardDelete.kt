package com.octaviantocan.mymind.ui.menu

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.text
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.library.UNDO_DELETE_MS
import kotlinx.coroutines.launch

/** How long Delete card is held before the card goes. */
const val DELETE_HOLD_MS = 1_300L

/**
 * Delete card as a row that is held, not tapped: a red fill and ring grow for [DELETE_HOLD_MS] and letting go early
 * keeps the card. Accessibility services delete with one action, since the Undo that follows covers a slip.
 *
 * @param onDelete Called once the hold completes.
 * @param onProgress How far the hold has come, from 0 to 1, each frame; 0 when it is let go or the row leaves.
 */
@Composable
internal fun HoldToDeleteRow(onDelete: () -> Unit, onProgress: (Float) -> Unit = {}) {
    val colors = menuColors()
    val haptic = rememberSheetHaptics()
    val feedback = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()
    val delete by rememberUpdatedState(onDelete)
    val report by rememberUpdatedState(onProgress)
    var progress by remember { mutableFloatStateOf(0f) }
    var holding by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { snapshotFlow { progress }.collect { report(it) } }
    DisposableEffect(Unit) { onDispose { report(0f) } }
    val danger = colors.dangerText
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp)
            .drawBehind { drawRect(danger.copy(alpha = 0.14f), size = Size(size.width * progress, size.height)) }
            .pointerInput(Unit) {
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false)
                    holding = true
                    haptic.tick()
                    val job = scope.launch {
                        // Frame time rather than an animation, so a phone with animations off still needs the whole hold.
                        val start = withFrameNanos { it }
                        while (progress < 1f) {
                            val now = withFrameNanos { it }
                            progress = ((now - start) / 1_000_000f / DELETE_HOLD_MS).coerceIn(0f, 1f)
                        }
                        feedback.performHapticFeedback(HapticFeedbackType.LongPress)
                        delete()
                    }
                    waitForUpOrCancellation()
                    if (progress < 1f) {
                        job.cancel()
                        holding = false
                        progress = 0f
                    }
                }
            }
            .semantics(mergeDescendants = true) {
                role = Role.Button
                onClick(label = "Delete card") { delete(); true }
            }
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Image(painterResource(R.drawable.deleteicon), null, Modifier.size(22.dp), colorFilter = ColorFilter.tint(danger))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(if (holding) "Keep holding to delete" else "Delete card",
                style = TextStyle(color = danger, fontSize = 17.sp, lineHeight = 22.sp, fontFamily = CardMenuFont))
            Text(if (holding) "Let go to keep it" else "Hold to delete",
                style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont))
        }
        if (holding) ProgressRing(progress, danger, 25.dp)
    }
}

/** A ring that fills clockwise from the top as [progress] goes from 0 to 1. */
@Composable
private fun ProgressRing(progress: Float, color: Color, size: Dp, track: Color = color.copy(alpha = 0.18f)) {
    Canvas(Modifier.size(size)) {
        val stroke = 2.5.dp.toPx()
        val inset = stroke / 2
        val arc = Size(this.size.width - stroke, this.size.height - stroke)
        drawArc(track, 0f, 360f, false, Offset(inset, inset), arc, style = Stroke(stroke))
        drawArc(color, -90f, 360f * progress, false, Offset(inset, inset), arc, style = Stroke(stroke, cap = StrokeCap.Round))
    }
}

/**
 * The bar that follows a delete: the card's name, then Undo with a ring that counts down the seconds it has left.
 *
 * @param name The deleted card's name.
 * @param onUndo Brings the card back.
 * @param modifier Modifier for the bar's place.
 * @param seconds How long Undo lasts.
 */
@Composable
fun UndoDeleteBar(name: String, onUndo: () -> Unit, modifier: Modifier = Modifier, seconds: Int = (UNDO_DELETE_MS / 1_000).toInt()) {
    val colors = menuColors()
    var left by remember { mutableFloatStateOf(1f) }
    LaunchedEffect(Unit) {
        val start = withFrameNanos { it }
        while (left > 0f) {
            val now = withFrameNanos { it }
            left = (1f - (now - start) / 1_000_000_000f / seconds).coerceIn(0f, 1f)
        }
    }
    val ink = colors.rowText
    val paper = colors.sheet
    Row(
        modifier.heightIn(min = 52.dp).clip(CircleShape).background(ink).padding(start = 18.dp, end = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // The name gives way to "deleted", so a long name still says what happened.
        Row(Modifier.weight(1f).clearAndSetSemantics { text = AnnotatedString("$name deleted"); liveRegion = LiveRegionMode.Polite },
            horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            val style = TextStyle(color = paper, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = CardMenuFont)
            Text(name, Modifier.weight(1f, fill = false), style = style, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text("deleted", style = style, maxLines = 1)
        }
        Row(
            Modifier.heightIn(min = 40.dp).clip(CircleShape).background(paper.copy(alpha = 0.14f))
                .clickable(role = Role.Button, onClickLabel = "Undo the delete", onClick = onUndo).padding(start = 12.dp, end = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("Undo", style = TextStyle(color = paper, fontSize = 15.sp, lineHeight = 21.sp, fontWeight = FontWeight.SemiBold, fontFamily = CardMenuFont))
            Box(Modifier.size(28.dp), contentAlignment = Alignment.Center) {
                ProgressRing(left, paper, 28.dp, track = paper.copy(alpha = 0.25f))
                Text("${kotlin.math.ceil(left * seconds).toInt()}", style = TextStyle(color = paper, fontSize = 13.sp, lineHeight = 18.sp, fontFamily = CardMenuFont))
            }
        }
    }
}
