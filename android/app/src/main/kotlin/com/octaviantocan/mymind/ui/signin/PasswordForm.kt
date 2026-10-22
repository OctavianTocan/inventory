package com.octaviantocan.mymind.ui.signin

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.ui.theme.MymindTheme

private const val SIGN_IN_LABEL = "Sign in"

/**
 * Username and password fields for accounts made with the server's user tool; the original app has no such form, so
 * it reuses the pill look of the Google button.
 *
 * @param username Current username text.
 * @param password Current password text.
 * @param onUsername Called with the new username text.
 * @param onPassword Called with the new password text.
 * @param onSubmit Called when the user submits a non-empty username and password.
 * @param modifier Modifier for the column.
 */
@Composable
fun PasswordForm(
    username: String,
    password: String,
    onUsername: (String) -> Unit,
    onPassword: (String) -> Unit,
    onSubmit: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val canSubmit = username.isNotBlank() && password.isNotEmpty()
    Column(modifier, verticalArrangement = Arrangement.spacedBy(8.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        PillField(username, onUsername, "Username", KeyboardOptions(imeAction = ImeAction.Next), VisualTransformation.None, {})
        PillField(
            password,
            onPassword,
            "Password",
            KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
            PasswordVisualTransformation(),
        ) { if (canSubmit) onSubmit() }
        val shape = RoundedCornerShape(64.dp)
        Box(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 48.dp)
                .background(MymindTheme.colors.accent.copy(alpha = if (canSubmit) 1f else 0.4f), shape)
                .clip(shape)
                .clickable(enabled = canSubmit, onClick = onSubmit),
            contentAlignment = Alignment.Center,
        ) {
            Text(SIGN_IN_LABEL, style = TextStyle(color = Color.White, fontSize = 18.sp, fontFamily = MymindTheme.fonts.body))
        }
    }
}

@Composable
private fun PillField(
    value: String,
    onValueChange: (String) -> Unit,
    hint: String,
    keyboardOptions: KeyboardOptions,
    transformation: VisualTransformation,
    onDone: () -> Unit,
) {
    val style = TextStyle(color = Color.Black, fontSize = 18.sp, fontFamily = MymindTheme.fonts.body)
    BasicTextField(
        value = value,
        onValueChange = onValueChange,
        singleLine = true,
        textStyle = style,
        cursorBrush = SolidColor(Color.Black),
        visualTransformation = transformation,
        keyboardOptions = keyboardOptions,
        keyboardActions = KeyboardActions(onDone = { onDone() }),
        decorationBox = { field ->
            Box(
                Modifier
                    .fillMaxWidth()
                    .heightIn(min = 48.dp)
                    .background(Color.White, RoundedCornerShape(64.dp))
                    .padding(horizontal = 30.dp, vertical = 10.dp),
                contentAlignment = Alignment.CenterStart,
            ) {
                if (value.isEmpty()) Text(hint, style = style.copy(color = MymindTheme.colors.muted))
                field()
            }
        },
        modifier = Modifier.fillMaxWidth(),
    )
}
