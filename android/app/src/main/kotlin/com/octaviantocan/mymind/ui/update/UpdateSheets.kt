package com.octaviantocan.mymind.ui.update

import android.content.Intent
import android.view.WindowManager
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import androidx.core.view.WindowCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * What the update screen's foot says about [step], in one short line, or null for none.
 *
 * @param step How far the update got.
 * @return The line.
 */
internal fun updateStepLine(step: UpdateStep): String? = when (step) {
    UpdateStep.Idle -> null
    UpdateStep.NeedsAllow -> "Allow Inventory to install updates first."
    UpdateStep.Preparing -> "Getting the update…"
    is UpdateStep.Downloading -> "Downloading ${(step.progress * 100).toInt()}%"
    UpdateStep.WaitingForUpload -> "Waiting for an upload to finish…"
    UpdateStep.Installing -> "Installing…"
    UpdateStep.Confirm -> "Confirm the update with Android."
    is UpdateStep.Failed -> step.reason
}

/**
 * The update screen's content: the version, what it brings over the installed one, and Update at its foot, which
 * shows how far the update got once tapped.
 *
 * @param model The offered release and the update's progress.
 * @param onUpdate Called for Update and Try again.
 * @param onContinue Called for Continue update, while Android waits for the user's approval.
 */
@Composable
fun UpdateGateContent(model: UpdateGateModel, onUpdate: () -> Unit, onContinue: () -> Unit) {
    val c = MymindTheme.palette.sheet
    val muted = MymindTheme.palette.sheetInk.muted
    val scroll = rememberScrollState()
    Column(
        Modifier.fillMaxSize().background(c.paper).safeDrawingPadding().padding(start = 16.dp, end = 16.dp, bottom = 24.dp),
    ) {
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(scroll).padding(top = 32.dp, bottom = 16.dp)) {
            Text(
                "Inventory ${model.release.versionName} is ready",
                Modifier.semantics { heading() },
                style = TextStyle(color = c.text, fontSize = 28.sp, lineHeight = 34.sp, fontWeight = FontWeight.SemiBold, fontFamily = MymindTheme.fonts.body),
            )
            Text(
                "Update to keep using Inventory. This is what it brings.",
                Modifier.padding(top = 8.dp),
                style = TextStyle(color = muted, fontSize = 16.sp, lineHeight = 24.sp, fontFamily = MymindTheme.fonts.body),
            )
            ChangelogEntries(model.entries, Modifier.padding(top = 20.dp))
        }
        UpdateFooter(model, c.text, c.paper, muted, onUpdate, onContinue)
    }
}

/** Update, Try again or Continue update with the reason above it, or how far a running update got. */
@Composable
private fun ColumnScope.UpdateFooter(model: UpdateGateModel, ink: Color, paper: Color, muted: Color, onUpdate: () -> Unit, onContinue: () -> Unit) {
    val step = model.step
    val small = TextStyle(color = muted, fontSize = 15.sp, lineHeight = 21.sp, fontFamily = MymindTheme.fonts.body)
    val busy = step is UpdateStep.Preparing || step is UpdateStep.Downloading || step is UpdateStep.WaitingForUpload || step is UpdateStep.Installing
    if (busy) {
        Row(Modifier.fillMaxWidth().height(52.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterHorizontally)) {
            if (step is UpdateStep.Downloading) {
                CircularProgressIndicator({ step.progress }, Modifier.size(26.dp), color = ink, strokeWidth = 3.dp, trackColor = ink.copy(alpha = 0.12f))
            } else {
                CircularProgressIndicator(Modifier.size(26.dp), color = ink, strokeWidth = 3.dp, trackColor = ink.copy(alpha = 0.12f))
            }
            Text(updateStepLine(step).orEmpty(), style = small.copy(color = ink), maxLines = 2)
        }
        return
    }
    updateStepLine(step)?.let {
        Text(it, Modifier.padding(bottom = 10.dp).align(Alignment.CenterHorizontally), style = small.copy(color = if (step is UpdateStep.Failed) MymindTheme.colors.danger else muted))
    }
    val label = when (step) {
        is UpdateStep.Failed -> "Try again"
        UpdateStep.Confirm -> "Continue update"
        else -> "Update to ${model.release.versionName}"
    }
    val shape = RoundedCornerShape(16.dp)
    Box(
        Modifier.fillMaxWidth().height(52.dp).clip(shape).background(ink)
            .clickable(role = Role.Button, onClick = if (step == UpdateStep.Confirm) onContinue else onUpdate),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = TextStyle(color = paper, fontSize = 17.sp, fontWeight = FontWeight.SemiBold, fontFamily = MymindTheme.fonts.body))
    }
}

/**
 * The update screen. While the server offers a newer release ThyMind shows only this, over everything, from the first
 * frame of a start. Nothing closes it but the update: Back leaves the app.
 *
 * @param model The offered release and the update's progress.
 * @param onUpdate Called for Update and Try again.
 * @param onContinue Called for Continue update.
 * @param onLeave Called for Back; sends the app to the background.
 */
@Composable
fun UpdateGate(model: UpdateGateModel, onUpdate: () -> Unit, onContinue: () -> Unit, onLeave: () -> Unit) {
    val paper = MymindTheme.palette.sheet.paper
    Dialog(
        onDismissRequest = onLeave,
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val view = LocalView.current
        SideEffect {
            val window = (view.parent as? DialogWindowProvider)?.window ?: return@SideEffect
            window.setDimAmount(0f)
            window.setWindowAnimations(0)
            window.clearFlags(WindowManager.LayoutParams.FLAG_DIM_BEHIND)
            window.isNavigationBarContrastEnforced = false
            val insets = WindowCompat.getInsetsController(window, view)
            insets.isAppearanceLightNavigationBars = paper.luminance() > 0.5f
            insets.isAppearanceLightStatusBars = paper.luminance() > 0.5f
        }
        UpdateGateContent(model, onUpdate, onContinue)
    }
}

/**
 * The sheet shown once after the app was updated.
 *
 * @param entry The notes of the version now running.
 * @param onClose Called when the sheet is closed.
 */
@Composable
fun AfterUpdateSheet(entry: ChangelogEntry, onClose: () -> Unit) {
    ChangelogScreen(listOf(entry), onClose)
}

/**
 * Everything the update feature draws over the signed-in app: the update screen, the after-update sheet and the
 * changelog sheet. The update screen hides the other two while it shows.
 *
 * @param viewModel State and actions of the update screens.
 * @param onAllow Opens Android's page to let the app install updates.
 * @param onLeave Sends the app to the background, for Back on the update screen.
 */
@Composable
fun UpdateOverlays(viewModel: UpdateViewModel, onAllow: () -> Unit, onLeave: () -> Unit) {
    val context = LocalContext.current
    val gate by viewModel.gate.collectAsStateWithLifecycle()
    val after by viewModel.afterUpdate.collectAsStateWithLifecycle()
    val changelog by viewModel.changelogOpen.collectAsStateWithLifecycle()
    val promptRequest by viewModel.promptRequest.collectAsStateWithLifecycle()
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    val openPrompt = { automatic: Boolean ->
        viewModel.promptToOpen(automatic)?.let { runCatching { context.startActivity(Intent(it)) } }
    }
    LaunchedEffect(promptRequest) {
        if (lifecycle.currentState.isAtLeast(Lifecycle.State.RESUMED)) openPrompt(true)
    }
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) { openPrompt(true) }
    gate?.let { model ->
        UpdateGate(
            model,
            onUpdate = { if (!viewModel.update()) onAllow() },
            onContinue = { openPrompt(false) },
            onLeave = onLeave,
        )
        return
    }
    if (changelog) ChangelogScreen(viewModel.entries, viewModel::closeChangelog)
    after?.let { AfterUpdateSheet(it, viewModel::dismissAfterUpdate) }
}
