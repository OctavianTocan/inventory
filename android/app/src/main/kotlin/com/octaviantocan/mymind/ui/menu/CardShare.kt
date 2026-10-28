package com.octaviantocan.mymind.ui.menu

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.siteOf

/**
 * The Share sheet's page: what of the card can be sent, as large choices, then Copy link. A null action leaves its
 * choice out.
 *
 * @param card Card being shared.
 * @param onShareLink Sends the card's link to another app.
 * @param onShareText Sends the card's own words to another app.
 * @param onCopyLink Puts the card's link on the clipboard.
 */
@Composable
internal fun ShareContent(card: Card, onShareLink: (() -> Unit)?, onShareText: (() -> Unit)?, onCopyLink: (() -> Unit)?) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 30.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        onShareLink?.let { SheetChoice(painterResource(R.drawable.ic_link), "The link", siteOf(card) ?: card.url, it) }
        onShareText?.let { SheetChoice(painterResource(R.drawable.ic_summary), "The words", "As text, to paste anywhere", it) }
        if (onCopyLink != null) {
            Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.Center) {
                SheetPill(painterResource(R.drawable.ic_link), "Copy link", onCopyLink)
            }
        }
    }
}
