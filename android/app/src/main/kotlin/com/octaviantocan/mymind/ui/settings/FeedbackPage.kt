package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.aikit.AiSettingsLogic
import com.octaviantocan.aikit.CliCatalog
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.AI_ENGINES
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.FeedbackProblem
import com.octaviantocan.mymind.data.FeedbackStatus
import com.octaviantocan.mymind.data.cli
import com.octaviantocan.mymind.data.isFinal
import com.octaviantocan.mymind.feedback.FeedbackTracker
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowChevron
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.UUID

/** How often the open page asks the server where its job stands, in milliseconds. */
const val FEEDBACK_POLL_MS = 20_000L

/** The longest feedback the server takes, in characters. */
const val FEEDBACK_MAX_CHARS = 4_000

/** The effort a feedback starts with when its model has no default of its own. */
private const val FEEDBACK_EFFORT = "medium"

/** The line under Send, saying whose plan runs the change and who gets it. */
const val FEEDBACK_CONSENT = "Uses your Claude or Codex plan. Your change ships to everyone using Inventory."

/** The engine line when neither Claude Code nor Codex is signed in. */
const val FEEDBACK_NO_ENGINE = "Sign in to Claude Code or Codex first"

/**
 * The "Send feedback" page: a text field, one engine line (engine, model, effort) that opens the model and effort
 * pages, Send, and the consent line under it. Only signed-in engines count, and feedback keeps its own engine, model
 * and effort ([AiAccount.feedbackPrefs]) apart from AI search, starting at the engine's first recommended model at
 * medium. While a job is not final the page shows its state and asks the server every [pollMs].
 *
 * @param ai The account's AI: its sign-ins, model lists and the feedback choice.
 * @param feedback Sends the feedback and follows its job.
 * @param open Opens a child page, such as the feedback model page.
 * @param onProviders Opens Providers, where the engines are signed in.
 * @param askNotifications Asks for the notification permission, if the app has not asked yet; called as feedback is sent.
 * @param pollMs How often to ask where the job stands while the page is open.
 */
@Composable
internal fun ColumnScope.FeedbackPage(
    ai: AiAccount,
    feedback: FeedbackTracker,
    open: (ProfilePage) -> Unit,
    onProviders: () -> Unit,
    askNotifications: () -> Unit = {},
    pollMs: Long = FEEDBACK_POLL_MS,
) {
    val scope = rememberCoroutineScope()
    val prefs = ai.feedbackPrefs
    val choice by prefs.choice.collectAsState()
    val signIns by ai.signIns.collectAsState()
    LaunchedEffect(ai) { runCatching { ai.refreshSignIns() } }
    val ready = AI_ENGINES.filter { id -> signIns?.let { it.account && it[id.cli].signedIn } == true }
    val engineReady = choice.engine in ready
    LaunchedEffect(ready) { if (ready.isNotEmpty() && choice.engine !in ready) prefs.chooseEngine(ready.first()) }
    var catalog by remember(choice.engine) { mutableStateOf(ai.cachedModels(choice.engine)) }
    LaunchedEffect(choice.engine, engineReady) {
        if (engineReady && catalog == null) catalog = runCatching { ai.models(choice.engine) }.getOrNull()
    }
    // Only a signed-in engine's own list fills the choice in; a list that arrives as the engine changes is left alone.
    LaunchedEffect(catalog, choice.engine, choice.model, engineReady) {
        val list = catalog ?: return@LaunchedEffect
        if (!engineReady || choice.model.isNotBlank()) return@LaunchedEffect
        val first = list.models.firstOrNull { it.isDefault } ?: list.models.firstOrNull() ?: return@LaunchedEffect
        prefs.chooseModel(first.id, list)
        prefs.chooseEffort(if (first.efforts.isEmpty() || FEEDBACK_EFFORT in first.efforts) FEEDBACK_EFFORT else "")
    }
    val model = AiSettingsLogic.current(catalog, choice.model)

    var jobId by remember { mutableStateOf(feedback.activeId()) }
    var status by remember { mutableStateOf<FeedbackStatus?>(null) }
    LaunchedEffect(jobId) {
        val id = jobId ?: return@LaunchedEffect
        while (true) {
            try {
                status = feedback.refresh(id)
            } catch (e: CancellationException) {
                throw e
            } catch (e: FeedbackProblem) {
                if (e.notFound) {
                    jobId = null
                    status = null
                    return@LaunchedEffect
                }
            }
            if (status?.isFinal == true) return@LaunchedEffect
            delay(pollMs)
        }
    }

    var text by remember { mutableStateOf("") }
    // A retry of the same send keeps the id, so the server answers with the job it already made; other words are another send.
    val clientId = remember(text) { UUID.randomUUID().toString() }
    var busy by remember { mutableStateOf(false) }
    var problem by remember { mutableStateOf<String?>(null) }
    val formOpen = status?.isFinal != false
    val sendable = formOpen && ready.isNotEmpty() && engineReady && choice.model.isNotBlank() && text.isNotBlank() && !busy

    fun send() {
        if (!sendable) return
        val sending = AiChoice(choice.engine, choice.model, choice.effort.ifBlank { model?.defaultEffort ?: FEEDBACK_EFFORT })
        busy = true
        problem = null
        askNotifications()
        scope.launch {
            try {
                val sent = feedback.send(text.trim(), sending, clientId)
                status = sent
                jobId = sent.id
                text = ""
            } catch (e: CancellationException) {
                throw e
            } catch (e: FeedbackProblem) {
                problem = e.message
            } finally {
                busy = false
            }
        }
    }

    status?.let { shown ->
        CardMenuGroup(Modifier.padding(top = 20.dp)) {
            SheetRow(null, shown.title.ifBlank { "Your feedback" }, null, detail = shown.detail.ifBlank { null })
        }
    }
    if (formOpen) {
        CardMenuGroup(Modifier.padding(top = if (status == null) 20.dp else 12.dp)) {
            FeedbackField(text, { if (it.length <= FEEDBACK_MAX_CHARS) text = it }, enabled = !busy)
        }
        if (signIns != null && ready.isEmpty()) {
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                SheetRow(null, FEEDBACK_NO_ENGINE, onProviders, trailing = { SheetRowChevron() })
            }
        } else if (ready.isNotEmpty()) {
            EngineRows(choice, catalog, model?.efforts?.size ?: 0, open)
        }
        PrimaryButton(if (busy) "Sending…" else "Send", enabled = sendable, onClick = ::send)
        Explain(FEEDBACK_CONSENT)
        problem?.let { Explain(it, error = true) }
    }
}

/** The engine line and, when the model has more than one effort, the Effort row under it. */
@Composable
private fun EngineRows(choice: AiChoice, catalog: CliCatalog?, efforts: Int, open: (ProfilePage) -> Unit) {
    val model = AiSettingsLogic.current(catalog, choice.model)
    val effort = choice.effort.ifBlank { model?.defaultEffort ?: FEEDBACK_EFFORT }
    val cli = choice.engine.cli
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        SheetRow(engineMark(cli), "${cli.title} · ${modelName(choice, catalog)} · ${effortName(effort)}", { open(ProfilePage.FeedbackModel) },
            iconTint = engineTint(cli), trailing = { SheetRowChevron() })
        if (efforts > 1) {
            SheetRowDivider()
            SheetRow(painterResource(R.drawable.ic_library_search_spark), "Effort", { open(ProfilePage.FeedbackEffort) },
                detail = effortName(effort), trailing = { SheetRowChevron() })
        }
    }
}

/** The multi-line field the feedback is typed into. */
@Composable
private fun FeedbackField(text: String, onChange: (String) -> Unit, enabled: Boolean) {
    val colors = menuColors()
    BasicTextField(
        value = text,
        onValueChange = onChange,
        enabled = enabled,
        textStyle = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont),
        cursorBrush = SolidColor(colors.rowText),
        keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences),
        modifier = Modifier.fillMaxWidth().semantics { contentDescription = "Feedback" },
        decorationBox = { field ->
            Box(Modifier.heightIn(min = 132.dp).padding(horizontal = 16.dp, vertical = 14.dp), contentAlignment = Alignment.TopStart) {
                if (text.isEmpty()) Text("What should Inventory change?",
                    style = TextStyle(color = colors.rowText.copy(alpha = 0.5f), fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont))
                field()
            }
        },
    )
}
