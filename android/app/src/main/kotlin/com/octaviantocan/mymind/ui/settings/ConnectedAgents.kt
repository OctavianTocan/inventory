package com.octaviantocan.mymind.ui.settings

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.data.ConnectedAgent
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

/** Label of the profile row and the page that list the connected AI apps. */
const val CONNECTED_AGENTS = "Connected agents"

/**
 * The Connected agents page: each AI app signed in through ThyMind's MCP server, when it connected and was last
 * used, and Remove. Remove asks once more on the same row ("Tap to remove") before it signs the app out, because the
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
        agents.isEmpty() -> Note("No AI apps are connected. Add Inventory to Claude, ChatGPT or another AI app, and it shows here.")
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
