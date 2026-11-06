package com.octaviantocan.mymind.ui.settings

import android.content.ClipData
import android.content.Intent
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.ConnectedAgent
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.menuColors
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlinx.coroutines.launch

/** Label of the profile row and the page that list the connected AI apps. */
const val CONNECTED_AGENTS = "Connected agents"

/**
 * The Connected agents page: each AI app signed in through ThyMind's MCP server, when it connected and was last
 * used, and Remove, then the ways to add Inventory to ChatGPT or Claude ([AddToAiApps]). Remove asks once more on the same row ("Tap to remove") before it signs the app out, because the
 * app must be connected again from its own settings afterwards.
 *
 * @param state The apps, or null while the server has not answered.
 * @param onLoad Called when the page opens, to ask the server again.
 * @param onRemove Called with an app's id once its removal is confirmed.
 * @param today The day "today" and "yesterday" count from.
 */
@Composable
internal fun ColumnScope.ConnectedAgentsPage(
    state: AgentsUiState,
    onLoad: () -> Unit,
    onRemove: (String) -> Unit,
    today: LocalDate = LocalDate.now(),
) {
    LaunchedEffect(Unit) { onLoad() }
    var confirming by remember { mutableStateOf<String?>(null) }
    val agents = state.agents
    when {
        agents == null && state.failed -> Note("Inventory did not answer. Try again in a moment.", error = true)
        agents == null -> Note("Loading…")
        agents.isEmpty() -> Note("No AI apps are connected yet. Once you add Inventory to one, it shows here.")
        else -> {
            CardMenuGroup(Modifier.padding(top = 20.dp)) {
                agents.forEachIndexed { index, agent ->
                    if (index > 0) SheetRowDivider(start = 16.dp)
                    val asking = confirming == agent.id
                    SheetRow(null, agent.name, onClick = {
                        if (asking) {
                            confirming = null
                            onRemove(agent.id)
                        } else {
                            confirming = agent.id
                        }
                    }, detail = agentDates(agent, today), trailing = {
                        Text(if (asking) "Tap to remove" else "Remove", style = TextStyle(fontFamily = CardMenuFont, fontSize = 15.sp,
                            color = if (asking) menuColors().dangerText else menuColors().rowText.copy(alpha = 0.65f)))
                    })
                }
            }
            if (state.failed) Note("Inventory did not remove that app. Try again in a moment.", error = true)
        }
    }
    AddToAiApps()
}

/** Where ChatGPT adds a custom MCP server (developers.openai.com/api/docs/guides/custom-mcp-server, 6 Nov 2026). */
internal const val CHATGPT_PLUGINS_URL = "https://chatgpt.com/plugins"

/** Where Claude adds a custom connector. */
internal const val CLAUDE_CONNECTORS_URL = "https://claude.ai/settings/connectors"

/**
 * The address an AI app adds to reach this account's library through Inventory's MCP server.
 *
 * @param server The Inventory server, [BuildConfig.SERVER_URL] in the app.
 * @return `<server>/mcp`.
 */
internal fun mcpServerUrl(server: String = BuildConfig.SERVER_URL): String = "${server.trimEnd('/')}/mcp"

/**
 * The steps that add Inventory to ChatGPT and Claude, as sent to another device by "Send the steps".
 *
 * @param mcpUrl The MCP server address, [mcpServerUrl] in the app.
 * @return Plain text naming both apps, each with its page and the address to paste.
 */
internal fun aiSetupSteps(mcpUrl: String = mcpServerUrl()): String = """
    |Add Inventory to an AI app
    |
    |ChatGPT (on the web): open $CHATGPT_PLUGINS_URL, tap +, then Add custom MCP server. Name it Inventory, paste $mcpUrl as the server URL, choose OAuth, create it, and sign in with Google.
    |
    |Claude: open $CLAUDE_CONNECTORS_URL, then Add custom connector. Name it Inventory, paste $mcpUrl, add it, then Connect and sign in with Google.
""".trimMargin()

/**
 * The Add to an AI app group: ChatGPT and Claude each copy the MCP address and open the page where that app adds it,
 * and Send the steps shares [aiSetupSteps] so the person can finish on a computer, where ChatGPT adds custom servers.
 */
@Composable
private fun AddToAiApps() {
    val clipboard = LocalClipboard.current
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val url = mcpServerUrl()
    val copyAndOpen = { app: String, page: String ->
        Track.event("ai app setup", detail = app)
        scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("Inventory MCP address", url))) }
        runCatching { uriHandler.openUri(page) }
        Unit
    }
    CardMenuGroup(Modifier.padding(top = 20.dp)) {
        SheetRow(painterResource(R.drawable.ic_si_openai), "Add to ChatGPT", { copyAndOpen("chatgpt", CHATGPT_PLUGINS_URL) },
            detail = "Copies the address and opens ChatGPT's plugins")
        SheetRowDivider()
        SheetRow(painterResource(R.drawable.ic_si_claude), "Add to Claude", { copyAndOpen("claude", CLAUDE_CONNECTORS_URL) },
            detail = "Copies the address and opens Claude's connectors")
        SheetRowDivider()
        SheetRow(painterResource(R.drawable.ic_share), "Send the steps", {
            Track.event("ai app setup", detail = "send steps")
            val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, aiSetupSteps(url))
            runCatching { context.startActivity(Intent.createChooser(send, null)) }
        }, detail = "To finish on a computer")
    }
    Note("In ChatGPT, tap +, then Add custom MCP server, paste the address and sign in with Google. ChatGPT adds it on the web, so a computer is easiest. In Claude, tap Add custom connector and paste it.")
}

/** Muted words on the page, or a problem in the destructive colour. */
@Composable
private fun Note(text: String, error: Boolean = false) {
    val colors = menuColors()
    Text(text, Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 20.dp),
        style = TextStyle(fontFamily = CardMenuFont, fontSize = 14.sp, lineHeight = 20.sp,
            color = if (error) colors.dangerText else colors.rowText.copy(alpha = 0.65f)))
}

/**
 * When an app connected and was last used, in the phone's time zone.
 *
 * @param agent The app.
 * @param today The day "today" and "yesterday" count from.
 * @return Such as `Connected 5 Nov · used today`, or `Connected today` when both fall on the same day.
 */
fun agentDates(agent: ConnectedAgent, today: LocalDate = LocalDate.now()): String {
    val connected = day(agent.connected) ?: return ""
    val used = day(agent.used)
    val connectedText = "Connected ${dayName(connected, today)}"
    return if (used == null || used == connected) connectedText else "$connectedText · used ${dayName(used, today)}"
}

private fun day(iso: String): LocalDate? = runCatching { Instant.parse(iso).atZone(ZoneId.systemDefault()).toLocalDate() }.getOrNull()

private fun dayName(day: LocalDate, today: LocalDate): String = when (day) {
    today -> "today"
    today.minusDays(1) -> "yesterday"
    else -> day.format(DateTimeFormatter.ofPattern(if (day.year == today.year) "d MMM" else "d MMM yyyy", Locale.ENGLISH))
}
