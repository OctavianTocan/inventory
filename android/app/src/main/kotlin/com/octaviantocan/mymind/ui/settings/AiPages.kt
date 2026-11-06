package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.aikit.AiCli
import com.octaviantocan.aikit.AiSettingsLogic
import com.octaviantocan.aikit.AiSignInServices
import com.octaviantocan.aikit.AiSignInStart
import com.octaviantocan.aikit.AiSignInStep
import com.octaviantocan.aikit.AiSignIns
import com.octaviantocan.aikit.CliCatalog
import com.octaviantocan.aikit.CliModel
import com.octaviantocan.aikit.DeviceSignInEnd
import com.octaviantocan.aikit.aiSignInStep
import com.octaviantocan.aikit.awaitDeviceSignIn
import com.octaviantocan.aikit.groupModels
import com.octaviantocan.aikit.isClaudeSignInCode
import com.octaviantocan.aikit.isCallbackFor
import com.octaviantocan.aikit.isCodeFor
import com.octaviantocan.aikit.modelLabel
import com.octaviantocan.aikit.modelNote
import com.octaviantocan.aikit.prettyModelId
import com.octaviantocan.aikit.sameNamed
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.AI_ENGINES
import com.octaviantocan.mymind.data.AiAccount
import com.octaviantocan.mymind.data.AiChoice
import com.octaviantocan.mymind.data.AiPrefs
import com.octaviantocan.mymind.data.CODEX
import com.octaviantocan.mymind.data.cli
import com.octaviantocan.mymind.ui.card.CardSheetIndication
import com.octaviantocan.mymind.ui.card.LocalCardSheetColors
import com.octaviantocan.mymind.ui.card.cardSurfaceShadow
import com.octaviantocan.mymind.ui.card.rememberSheetHaptics
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.CardMenuHeading
import com.octaviantocan.mymind.ui.menu.SheetPage
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowChevron
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors
import com.octaviantocan.mymind.ui.shell.LoadingSpinner
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

/** The Profile sheet's child pages. */
enum class ProfilePage(override val title: String) : SheetPage {
    ClaudeSignIn(AiCli.Claude.title),
    CodexSignIn(AiCli.Codex.title),
    Model("Search model"),
    Effort("Search effort"),
    Feedback("Send feedback"),
    FeedbackModel("Feedback model"),
    FeedbackEffort("Feedback effort"),
    Agents(CONNECTED_AGENTS),
    Theme("Theme"),
    ;

    override val depth: Int get() = 1
}

/** The sign-in page of a CLI. */
val AiCli.page: ProfilePage get() = if (this == AiCli.Claude) ProfilePage.ClaudeSignIn else ProfilePage.CodexSignIn

/** Pully's English names for the efforts (`ai.effort.*`). */
private val EFFORT_NAMES = mapOf(
    "ai.effort.minimal" to "Minimal", "ai.effort.low" to "Low", "ai.effort.medium" to "Medium", "ai.effort.high" to "High",
    "ai.effort.xhigh" to "Extra High", "ai.effort.max" to "Max", "ai.effort.ultra" to "Ultra",
)

/**
 * The name of an effort, as Pully shows it.
 *
 * @param id An effort id, such as `xhigh`.
 * @return Its name, such as "Extra High".
 */
fun effortName(id: String): String = AiSettingsLogic.effortLabel(id) { EFFORT_NAMES[it] ?: it }

/**
 * The name of the model a choice runs, as the model rows show it.
 *
 * @param choice The engine, model and effort.
 * @param catalog The engine's model list, when known.
 * @return The model's name, or "Default" for the CLI's own choice before its list is known.
 */
fun modelName(choice: AiChoice, catalog: CliCatalog?): String =
    AiSettingsLogic.current(catalog, choice.model)?.let(::modelLabel) ?: choice.model.ifBlank { null }?.let(::prettyModelId) ?: "Default"

/** Claude's and OpenAI's marks, Claude's in its own colour as on Pully's rows. */
@Composable
internal fun engineMark(cli: AiCli): Painter = painterResource(if (cli == AiCli.Codex) R.drawable.ic_si_openai else R.drawable.ic_si_claude)

internal fun engineTint(cli: AiCli): Color? = if (cli == AiCli.Claude) Color(0xFFD97757) else null

/**
 * The AI rows of the Profile sheet's root, laid out as Pully's Settings has them: the search model with its effort
 * under it, then each CLI's own sign-in under Providers. The rows show the last answers at once and ask again behind
 * them, so a row is never blank while it loads.
 *
 * @param ai The account's AI.
 * @param feedback True to end with the "Send feedback" row that opens [ProfilePage.Feedback].
 * @param open Opens a child page.
 */
@Composable
internal fun AiRows(ai: AiAccount, feedback: Boolean = false, open: (ProfilePage) -> Unit) {
    val signIns by ai.signIns.collectAsState()
    val choice by ai.prefs.choice.collectAsState()
    var catalog by remember(choice.engine) { mutableStateOf(ai.cachedModels(choice.engine)) }
    LaunchedEffect(ai) { runCatching { ai.refreshSignIns() } }
    val signedIn = signIns?.let { it.account && it[choice.engine.cli].signedIn } == true
    LaunchedEffect(choice.engine, signedIn) {
        if (signedIn && catalog == null) catalog = runCatching { ai.models(choice.engine) }.getOrNull()
    }
    val model = AiSettingsLogic.current(catalog, choice.model)
    // A model with one effort has nothing to choose, so it has no Effort row.
    val effortRow = (model?.efforts?.size ?: 0) > 1
    CardMenuHeading("AI search")
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        SheetRow(engineMark(choice.engine.cli), "Model", { open(ProfilePage.Model) }, detail = modelName(choice, catalog),
            iconTint = engineTint(choice.engine.cli), trailing = { SheetRowChevron() })
        if (effortRow) {
            SheetRowDivider()
            val effort = choice.effort.ifBlank { model?.defaultEffort.orEmpty() }
            SheetRow(painterResource(R.drawable.ic_library_search_spark), "Effort", { open(ProfilePage.Effort) },
                detail = effort.ifBlank { null }?.let(::effortName), trailing = { SheetRowChevron() })
        }
    }
    CardMenuHeading("Providers")
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        AiCli.entries.forEachIndexed { index, cli ->
            if (index > 0) SheetRowDivider()
            SheetRow(engineMark(cli), cli.title, { open(cli.page) }, detail = signIns?.let { if (it[cli].signedIn) "Signed in" else "Not signed in" },
                iconTint = engineTint(cli), trailing = { SheetRowChevron() })
        }
    }
    if (feedback) {
        CardMenuHeading("Feedback")
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(painterResource(R.drawable.ic_comment), "Send feedback", { open(ProfilePage.Feedback) }, trailing = { SheetRowChevron() })
        }
    }
}

/** What a sign-in page holds while it is open. */
internal class AiSignInState {
    var signIns by mutableStateOf<AiSignIns?>(null)
    var started by mutableStateOf<AiSignInStart?>(null)
    var code by mutableStateOf("")
    var busy by mutableStateOf(false)
    var problem by mutableStateOf<String?>(null)
    /** The address the browser came back to from Sign in with ChatGPT, kept until the app is in front to finish. */
    var returned by mutableStateOf<String?>(null)
    fun clear() { signIns = null; started = null; code = ""; busy = false; problem = null; returned = null }
}

/**
 * Pully's sign-in page for [cli], drawn with ThyMind's sheet parts: the CLI's own sign-in on the person's own
 * subscription. Claude Code's page shows a code that is pasted back here, or taken from the clipboard on the way
 * back; Codex signs in with OpenAI's Sign in with ChatGPT, whose page sends the browser back to the phone's own
 * loopback address, where the app takes the address and finishes.
 *
 * @param services The server, browser and clipboard.
 * @param cli The CLI to sign in.
 * @param notify Shows a short notice.
 */
@Composable
internal fun ColumnScope.AiSignInPage(services: AiSignInServices, cli: AiCli, notify: (String) -> Unit) {
    val colors = menuColors()
    val scope = rememberCoroutineScope()
    val state = remember(services, cli) { AiSignInState().apply { signIns = services.cached() } }
    val dispatchReady = services.dispatchReady
    DisposableEffect(services, state) {
        val stop = services.onRetired(state::clear)
        onDispose { stop(); state.clear() }
    }
    suspend fun refresh() {
        state.signIns = services.status()
    }
    fun run(work: suspend () -> Unit) {
        if (state.busy || !services.dispatchReady) return
        state.busy = true
        state.problem = null
        scope.launch {
            try {
                work()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                if (services.current()) state.problem = e.message ?: "Something went wrong. Try again."
            } finally { state.busy = false }
        }
    }
    fun signedIn() {
        state.started = null
        state.code = ""
        state.returned = null
        notify("${cli.title} is signed in")
    }
    LaunchedEffect(services, cli, dispatchReady) {
        if (!dispatchReady) return@LaunchedEffect
        try { refresh() } catch (e: CancellationException) { throw e } catch (e: Exception) { state.problem = e.message }
    }
    // Codex finishes by itself once its code is entered on OpenAI's page.
    val waiting = state.started?.takeIf { !it.paste }
    LaunchedEffect(waiting, dispatchReady) {
        if (waiting == null || !dispatchReady) return@LaunchedEffect
        when (val end = services.awaitDeviceSignIn(cli)) {
            DeviceSignInEnd.Done -> { refresh(); signedIn() }
            // The sign-in itself is what counts, even when the wait for it said it ended.
            is DeviceSignInEnd.FinishedMeanwhile -> { state.started = null; state.signIns = end.signIns; signedIn() }
            is DeviceSignInEnd.Failed -> { state.started = null; state.problem = end.message }
            DeviceSignInEnd.Stopped -> Unit
        }
    }

    fun finishPasted() = run { if (services.finish(cli, state.code)) { refresh(); signedIn() } }

    // Sign in with ChatGPT: the app listens on the callback while the browser is in front, and finishes once it is back.
    val browsing = state.started?.takeIf { it.callback != null }
    LaunchedEffect(browsing) {
        val callback = browsing?.callback ?: return@LaunchedEffect
        try {
            val address = services.awaitCallback(callback)
            if (state.started === browsing && isCallbackFor(browsing, address)) state.returned = address
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            if (services.current() && state.started === browsing) {
                state.problem = e.message?.takeIf { "too long" in it } ?: "Inventory couldn't wait for ChatGPT here. When the page ends, copy its address and paste it below."
            }
        }
    }
    LaunchedEffect(state.returned, dispatchReady) {
        val address = state.returned ?: return@LaunchedEffect
        if (!dispatchReady) return@LaunchedEffect
        state.returned = null
        run { if (services.finish(cli, address)) { refresh(); signedIn() } }
    }
    // Back from Claude's page with its code copied, the app takes the code itself and finishes, so nothing is pasted by
    // hand (Pully, recording 26 Oct 2026, 02:59 to 03:13).
    LaunchedEffect(state.started, dispatchReady) {
        val pasting = state.started?.takeIf { it.paste } ?: return@LaunchedEffect
        if (!dispatchReady || state.code.isNotBlank() || state.busy) return@LaunchedEffect
        val copied = services.copied()?.trim()
        if (state.started === pasting && isCodeFor(pasting.url, copied)) {
            state.code = copied!!
            finishPasted()
        }
    }

    val signIns = state.signIns
    val signIn = signIns?.get(cli)
    val started = state.started
    when (aiSignInStep(cli, signIns, started, state.problem)) {
        AiSignInStep.Loading -> Box(Modifier.fillMaxWidth().padding(vertical = 32.dp), contentAlignment = Alignment.Center) {
            LoadingSpinner(color = colors.rowText)
        }
        AiSignInStep.Unavailable -> Unit
        AiSignInStep.NeedsAccount -> Explain("Sign in to Inventory first, so ${cli.title} runs on your own subscription and nobody else's.")
        AiSignInStep.PasteCode -> {
            started!!
            Explain("Sign in on the page that opened. When it shows a code, copy it and paste it here.")
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                CodeField(state.code, { typed ->
                    state.code = typed.trim()
                    // A whole code pasted in finishes at once; there is no button to find under the keyboard.
                    if (isClaudeSignInCode(state.code)) finishPasted()
                }, enabled = !state.busy, onDone = { if (state.code.isNotBlank()) finishPasted() })
                SheetRowDivider(start = 16.dp)
                SheetRow(null, "Open the sign-in page again", { services.openPage(started.url) })
            }
            PrimaryButton(if (state.busy) "Signing in…" else "Finish signing in", enabled = state.code.isNotBlank() && !state.busy, onClick = ::finishPasted)
            Cancel(services, cli, state, ::run)
        }
        AiSignInStep.EnterCode -> {
            started!!
            Explain("Enter this code on the page that opened. Inventory finishes by itself once you approve.")
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                SheetRow(null, started.code.orEmpty(), { started.code?.let { services.copy(it); notify("Code copied") } }) {
                    Text("Copy", style = rowValue(colors.rowText))
                }
                SheetRowDivider(start = 16.dp)
                SheetRow(null, "Open the sign-in page again", { services.openPage(started.url) })
            }
            Explain("If OpenAI says device codes are off, turn on device code sign-in in ChatGPT's security settings, then start again.")
            Cancel(services, cli, state, ::run)
        }
        AiSignInStep.Browser -> {
            started!!
            Explain("Sign in with your ChatGPT account on the page that opened and allow Inventory. Come back here when it says ChatGPT is connected.")
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                SheetRow(null, "Open the sign-in page again", { services.openPage(started.url) })
            }
            Explain("If the last page doesn't load, copy its address and paste it here.")
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                CodeField(state.code, { typed ->
                    state.code = typed.trim()
                    if (isCallbackFor(started, state.code)) finishPasted()
                }, enabled = !state.busy, onDone = { if (state.code.isNotBlank()) finishPasted() }, placeholder = "Paste the address")
            }
            if (state.busy) Explain("Signing in…")
            Cancel(services, cli, state, ::run)
        }
        AiSignInStep.SignedIn -> {
            signIn!!
            CardMenuGroup(Modifier.padding(top = 12.dp)) {
                SheetRow(null, "Signed in", null) {
                    // The account takes what room is left and ends in "…"; "Signed in" never wraps.
                    listOfNotNull(signIn.email, signIn.planName).joinToString(" · ").takeIf { it.isNotEmpty() }?.let {
                        Text(it, Modifier.weight(1f, fill = false), style = rowValue(colors.rowText), maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
            }
            Explain("AI search for your account runs on this sign-in and counts against your plan.")
            PrimaryButton(if (state.busy) "Signing out…" else "Sign out", enabled = !state.busy, danger = true) {
                run { services.signOut(cli); refresh() }
            }
        }
        AiSignInStep.SignedOut -> {
            Explain(if (cli == AiCli.Claude) "Inventory runs Claude Code with your own Claude subscription. Sign in once with your Claude account."
                else "Inventory runs Codex on your own ChatGPT plan, with OpenAI's Sign in with ChatGPT. Sign in once with your ChatGPT account.")
            PrimaryButton(if (state.busy) "Starting…" else if (cli == AiCli.Codex) "Sign in with ChatGPT" else "Sign in to ${cli.title}", enabled = !state.busy) {
                run {
                    val start = services.start(cli)
                    state.code = ""
                    state.returned = null
                    state.started = start
                    // Codex's page asks for the code shown here, so it is on the clipboard before the page opens and
                    // only needs pasting there.
                    start.code?.takeIf { !start.paste }?.let {
                        services.copy(it)
                        notify("Code copied")
                    }
                    services.openPage(start.url)
                }
            }
        }
    }
    state.problem?.let { Explain(it, error = true) }
}

/**
 * Pully's model page: the engine tabs, the newest models, and the older ones under their own heading. A row is a
 * name with a "Recommended" pill on the CLI's own pick and a plain ✓ on the one in use. Switching engine swaps the
 * lists where they stand. The sheet's header names the page ([ProfilePage.title]), so search and feedback differ there.
 *
 * @param ai The account's AI, whose sign-ins and model lists the page reads.
 * @param prefs The choice the page edits: [AiAccount.prefs] for AI search, [AiAccount.feedbackPrefs] for feedback.
 * @param open Opens another page, such as a sign-in page.
 * @param onDone Called once a model is chosen.
 */
@Composable
internal fun ColumnScope.ModelPage(ai: AiAccount, prefs: AiPrefs, open: (ProfilePage) -> Unit, onDone: () -> Unit) {
    val colors = menuColors()
    val choice by prefs.choice.collectAsState()
    val signIns by ai.signIns.collectAsState()
    val engine = choice.engine
    var attempt by remember { mutableIntStateOf(0) }
    var loaded by remember(engine) { mutableStateOf(ai.cachedModels(engine)?.let { Result.success(it) }) }
    val signedOut = { id: String -> signIns?.let { !it[id.cli].signedIn } ?: false }
    LaunchedEffect(engine, attempt, signedOut(engine)) {
        if (signedOut(engine) || (attempt == 0 && loaded?.isSuccess == true)) return@LaunchedEffect
        loaded = null
        loaded = try { Result.success(ai.models(engine)) } catch (e: CancellationException) { throw e } catch (e: Exception) { Result.failure(e) }
    }
    val catalog = loaded?.getOrNull()
    // An id the server now lists under another one ticks the row of the same name.
    val inUse = catalog?.let { AiSettingsLogic.current(it, choice.model)?.id ?: sameNamed(it.models, choice.model)?.id ?: choice.model }
    val groups = catalog?.let { groupModels(it.models) }
    fun choose(model: CliModel) {
        val chosen = catalog ?: return
        prefs.chooseModel(model.id, chosen)
        onDone()
    }
    EngineTabs(engine, enabled = { !signedOut(it) }, onSelect = prefs::chooseEngine)
    if (signedOut(engine)) {
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            SheetRow(engineMark(engine.cli), "${engine.cli.title} is not signed in", { open(engine.cli.page) },
                detail = "Sign it in to choose its model", iconTint = engineTint(engine.cli), trailing = { SheetRowChevron() })
        }
        return
    }
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        when {
            loaded == null -> Box(Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
                LoadingSpinner(color = colors.rowText)
            }
            catalog == null || groups == null -> {
                SheetRow(null, "Couldn't load the model list", null, detail = loaded?.exceptionOrNull()?.message)
                SheetRowDivider(start = 16.dp)
                SheetRow(null, "Try again", { attempt++ })
            }
            else -> groups.recent.forEachIndexed { index, model ->
                if (index > 0) SheetRowDivider(start = 16.dp)
                ModelRow(model, model.id == inUse) { choose(model) }
            }
        }
    }
    // The rest sit under a small heading, as in Claude's picker, and are always there to scroll to.
    if (groups != null && groups.older.isNotEmpty()) {
        CardMenuHeading("Older models")
        CardMenuGroup(Modifier.padding(top = 12.dp)) {
            groups.older.forEachIndexed { index, model ->
                if (index > 0) SheetRowDivider(start = 16.dp)
                ModelRow(model, model.id == inUse) { choose(model) }
            }
        }
    }
}

/**
 * The efforts the chosen model supports, "Recommended" on its default and ✓ on the one in use. The sheet's header
 * names the page ([ProfilePage.title]).
 *
 * @param ai The account's AI, whose model lists the page reads.
 * @param prefs The choice the page edits: [AiAccount.prefs] for AI search, [AiAccount.feedbackPrefs] for feedback.
 * @param onDone Called once an effort is chosen.
 */
@Composable
internal fun ColumnScope.EffortPage(ai: AiAccount, prefs: AiPrefs, onDone: () -> Unit) {
    val colors = menuColors()
    val choice by prefs.choice.collectAsState()
    var catalog by remember(choice.engine) { mutableStateOf(ai.cachedModels(choice.engine)) }
    LaunchedEffect(choice.engine) { if (catalog == null) catalog = runCatching { ai.models(choice.engine) }.getOrNull() }
    val model = AiSettingsLogic.current(catalog, choice.model)
    val current = choice.effort.ifBlank { model?.defaultEffort.orEmpty() }
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        if (catalog == null) {
            Box(Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) { LoadingSpinner(color = colors.rowText) }
        }
        model?.efforts.orEmpty().forEachIndexed { index, id ->
            if (index > 0) SheetRowDivider(start = 16.dp)
            SheetRow(null, effortName(id), { prefs.chooseEffort(if (id == model?.defaultEffort) "" else id); onDone() }, role = Role.RadioButton) {
                if (id == model?.defaultEffort) RecommendedPill()
                Tick(id == current)
            }
        }
    }
    Explain("Higher effort ranks more carefully, but it takes longer and uses your limits faster.")
}

/** Pully's CapsuleTabs for the engines: each maker's mark and name, the chosen one raised. */
@Composable
private fun EngineTabs(selected: String, enabled: (String) -> Boolean, onSelect: (String) -> Unit) {
    val colors = menuColors()
    val sheet = LocalCardSheetColors.current
    val haptic = rememberSheetHaptics()
    Row(Modifier.fillMaxWidth().padding(top = 20.dp).cardSurfaceShadow(CircleShape, note = true).clip(CircleShape).background(colors.row).padding(4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        AI_ENGINES.forEach { engine ->
            val on = engine == selected
            val usable = enabled(engine)
            Row(Modifier.weight(1f).heightIn(min = 44.dp).clip(CircleShape).background(if (on) sheet.active else Color.Transparent)
                .semantics { this.selected = on }
                .clickable(remember { MutableInteractionSource() }, CardSheetIndication, enabled = usable && !on, role = Role.Tab) { haptic.tick(); onSelect(engine) }
                .padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically) {
                // The maker's name alone fits half the sheet beside its mark; the rows name the engine in full.
                Image(engineMark(engine.cli), null, Modifier.size(20.dp), colorFilter = ColorFilter.tint(engineTint(engine.cli) ?: colors.rowText),
                    alpha = if (usable) 1f else 0.4f)
                Text(if (engine == CODEX) "Codex" else "Claude", style = TextStyle(fontFamily = CardMenuFont, fontSize = 16.sp, lineHeight = 22.sp,
                    fontWeight = FontWeight.Medium, color = colors.rowText.copy(alpha = if (usable) 1f else 0.4f)))
            }
        }
    }
}

/** A model: its name, "Recommended" on the CLI's own pick, a note under it when it has one, and ✓ on the one in use. */
@Composable
private fun ModelRow(model: CliModel, selected: Boolean, onClick: () -> Unit) {
    SheetRow(null, modelLabel(model), onClick, detail = modelNote(model), role = Role.RadioButton) {
        if (model.isDefault) RecommendedPill()
        Tick(selected)
    }
}

/** `Recommended`: the pill on the choice a CLI makes for you, 26dp tall, 13sp medium, neutral, as Pully's is. */
@Composable
private fun RecommendedPill() {
    val colors = menuColors()
    Box(Modifier.height(26.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.08f)).padding(horizontal = 10.dp),
        contentAlignment = Alignment.Center) {
        Text("Recommended", style = TextStyle(fontFamily = CardMenuFont, fontSize = 13.sp, fontWeight = FontWeight.Medium, color = colors.rowText), maxLines = 1)
    }
}

/** The ✓ of a chosen row, in a slot of its own so a row never shifts when it appears. */
@Composable
internal fun Tick(shown: Boolean) {
    Box(Modifier.size(24.dp), contentAlignment = Alignment.Center) {
        if (shown) Image(painterResource(R.drawable.checkmark), "Chosen", Modifier.size(22.dp), colorFilter = ColorFilter.tint(menuColors().rowText))
    }
}

/** The field Claude Code's code is pasted into. */
@Composable
private fun CodeField(code: String, onChange: (String) -> Unit, enabled: Boolean, onDone: () -> Unit, placeholder: String = "Paste the code") {
    val colors = menuColors()
    BasicTextField(
        value = code,
        onValueChange = onChange,
        enabled = enabled,
        singleLine = true,
        textStyle = TextStyle(color = colors.rowText, fontSize = 17.sp, lineHeight = 24.sp, fontFamily = CardMenuFont),
        cursorBrush = SolidColor(colors.rowText),
        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { onDone() }),
        modifier = Modifier.fillMaxWidth().semantics { contentDescription = "Code" },
        decorationBox = { field ->
            Box(Modifier.heightIn(min = 52.dp).padding(horizontal = 16.dp, vertical = 14.dp), contentAlignment = Alignment.CenterStart) {
                if (code.isEmpty()) Text(placeholder, style = TextStyle(color = colors.rowText.copy(alpha = 0.5f), fontSize = 17.sp, fontFamily = CardMenuFont))
                field()
            }
        },
    )
}

/** The page's one action: a full-width pill in the row ink, or in the destructive colour for Sign out. */
@Composable
internal fun PrimaryButton(label: String, enabled: Boolean, danger: Boolean = false, onClick: () -> Unit) {
    val colors = menuColors()
    val fill = if (danger) colors.dangerText else colors.rowText
    val haptic = rememberSheetHaptics()
    Box(Modifier.padding(top = 20.dp).fillMaxWidth().heightIn(min = 48.dp).clip(RoundedCornerShape(100.dp))
        .background(fill.copy(alpha = if (enabled) 1f else 0.4f))
        .clickable(enabled = enabled, role = Role.Button) { haptic.tick(); onClick() }
        .padding(horizontal = 18.dp, vertical = 8.dp), contentAlignment = Alignment.Center) {
        Text(label, style = TextStyle(color = colors.sheet, fontSize = 16.sp, fontFamily = CardMenuFont, fontWeight = FontWeight.Medium))
    }
}

/** A page's Cancel, in a group of its own under the sign-in. */
@Composable
private fun Cancel(services: AiSignInServices, cli: AiCli, state: AiSignInState, run: (suspend () -> Unit) -> Unit) {
    CardMenuGroup(Modifier.padding(top = 12.dp)) {
        SheetRow(null, "Cancel", { state.started = null; state.code = ""; run { services.cancel(cli) } })
    }
}

/** Muted words under a group, or the last problem in the destructive colour. */
@Composable
internal fun Explain(text: String, error: Boolean = false) {
    val colors = menuColors()
    Text(text, Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 12.dp),
        style = TextStyle(fontFamily = CardMenuFont, fontSize = 14.sp, lineHeight = 20.sp, textAlign = TextAlign.Start,
            color = if (error) colors.dangerText else colors.rowText.copy(alpha = 0.65f)))
}

/** A row's muted value at its end. */
private fun rowValue(ink: Color) = TextStyle(fontFamily = CardMenuFont, fontSize = 14.sp, lineHeight = 20.sp, color = ink.copy(alpha = 0.65f))
