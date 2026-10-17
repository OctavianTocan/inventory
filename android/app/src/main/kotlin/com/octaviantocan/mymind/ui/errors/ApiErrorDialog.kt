package com.octaviantocan.mymind.ui.errors

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable

private const val CONFIRM_LABEL = "OK"

/**
 * Stock Material 3 alert dialog that reports a failed call.
 *
 * @param title Heading; omitted when null.
 * @param message Body text; omitted when null.
 * @param onDismiss Called when the user confirms or dismisses the dialog.
 */
@Composable
fun ApiErrorDialog(title: String?, message: String?, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = { TextButton(onClick = onDismiss) { Text(CONFIRM_LABEL) } },
        title = title?.let { { Text(it) } },
        text = message?.let { { Text(it) } },
    )
}
