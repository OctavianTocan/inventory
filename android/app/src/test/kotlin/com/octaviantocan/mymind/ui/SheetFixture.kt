package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.menu.CardMenuPanel

/** A PullySheet body in the bare panel, without its dialog window, so a screenshot of the root shows it. */
@Composable
fun SheetFixture(title: String, content: @Composable ColumnScope.() -> Unit) {
    CardMenuPanel(title, onDismiss = {}, closeLabel = "Close") {
        Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 30.dp), content = content)
    }
}
