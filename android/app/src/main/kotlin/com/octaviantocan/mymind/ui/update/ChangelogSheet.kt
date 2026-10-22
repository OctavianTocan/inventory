package com.octaviantocan.mymind.ui.update

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.changelog.ChangelogEntry
import com.octaviantocan.mymind.changelog.ChangelogPicture
import com.octaviantocan.mymind.ui.card.DarkCardSheetColors
import com.octaviantocan.mymind.ui.card.LightCardSheetColors
import com.octaviantocan.mymind.ui.menu.PullySheet
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** Pully's What's new presentation, shared by Settings and the once-after-install notice. */
@Composable
fun ChangelogScreen(entries: List<ChangelogEntry>, onClose: () -> Unit) {
    PullySheet("What's new", onClose) { ChangelogEntries(entries) }
}

/** Pully's version blocks, badges, summaries, group labels, bullet spacing and separators. */
@Composable
internal fun ChangelogEntries(entries: List<ChangelogEntry>, modifier: Modifier = Modifier) {
    val dark = MymindTheme.colors.dark
    val c = if (dark) DarkCardSheetColors else LightCardSheetColors
    val muted = if (dark) Color(0xFF9D9D9D) else Color(0xFF6E8285)
    val border = if (dark) Color(0xFF242424) else Color(0xFFECEBE7)
    val accent = if (dark) Color(0xFF4FB7C1) else Color(0xFF24929C)
    val read = TextStyle(fontFamily = MymindTheme.fonts.body, fontSize = 16.sp, lineHeight = 24.sp, color = c.text)
    val small = read.copy(fontSize = 15.sp, lineHeight = 21.sp, color = muted)
    val line = with(LocalDensity.current) { read.lineHeight.toDp() }
    Column(modifier.fillMaxWidth().padding(top = 8.dp)) {
        if (entries.isEmpty()) Text("Nothing is written down for this version.", style = read, color = muted)
        entries.forEachIndexed { index, entry ->
            Column(Modifier.fillMaxWidth()
                .then(if (index > 0) Modifier.drawBehind { drawRect(border, size = Size(size.width, 1.dp.toPx())) }.padding(top = 24.dp) else Modifier)
                .padding(bottom = 28.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(entry.versionName, Modifier.semantics { heading() }, style = small.copy(lineHeight = 22.sp, fontWeight = FontWeight.SemiBold))
                    if (index == 0) Text("New", Modifier.clip(CircleShape)
                        .background(if (dark) Color(0xFF14363B) else Color(0xFFE6F1F2)).padding(horizontal = 8.dp, vertical = 1.dp),
                        style = small.copy(fontWeight = FontWeight.Medium, color = if (dark) Color(0xFF9AD3D6) else Color(0xFF165E66)))
                }
                if (entry.summary.isNotBlank()) Text(entry.summary, Modifier.padding(top = 6.dp),
                    style = read.copy(fontSize = 18.sp, lineHeight = 26.sp, fontWeight = FontWeight.SemiBold))
                entry.pictures.filter { it.after == -1 }.forEach { ChangelogImage(it, border, Modifier.padding(top = 14.dp)) }
                var at = 0
                entry.groups.forEach { group ->
                    group.title?.let { Text(it, Modifier.padding(top = 18.dp).semantics { heading() }, style = small.copy(fontWeight = FontWeight.SemiBold)) }
                    Column(Modifier.padding(top = if (group.title != null) 8.dp else 14.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        group.items.forEach { item ->
                            val bullet = at++
                            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                Box(Modifier.height(line), contentAlignment = Alignment.Center) {
                                    Box(Modifier.size(6.dp).clip(CircleShape).background(accent))
                                }
                                Text(item, Modifier.weight(1f), style = read)
                            }
                            entry.pictures.filter { it.after == bullet }.forEach { ChangelogImage(it, border, Modifier.padding(start = 18.dp)) }
                        }
                    }
                }
            }
        }
    }
}

/** A bundled release illustration, under its own bullet with Pully's rounded hairline frame. */
@Composable
private fun ChangelogImage(picture: ChangelogPicture, border: Color, modifier: Modifier = Modifier) {
    val shape = RoundedCornerShape(12.dp)
    AsyncImage("file:///android_asset/${picture.path}", contentDescription = picture.alt,
        modifier = modifier.padding(top = 4.dp).fillMaxWidth().heightIn(max = 420.dp).clip(shape).border(1.dp, border, shape),
        contentScale = ContentScale.Fit)
}
