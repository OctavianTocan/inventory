package com.octaviantocan.mymind.ui.share

import androidx.annotation.DrawableRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.share.MAX_SHARE_ITEMS
import com.octaviantocan.mymind.share.ShareState
import com.octaviantocan.mymind.ui.menu.CardMenuFont
import com.octaviantocan.mymind.ui.menu.menuColors

/**
 * Words of a sheet that says why nothing was saved.
 *
 * @property icon Its mark.
 * @property title What went wrong.
 * @property body What to do about it.
 * @property button The button's words.
 */
class ProblemText(@DrawableRes val icon: Int, val title: String, val body: String, val button: String)

/**
 * The words the share target shows for a state (share-problems-speak-for-thymind).
 *
 * @param problem The state.
 * @return Mark, title, body and button, or null for a state that is not a problem.
 */
fun problemText(problem: ShareState): ProblemText? = when (problem) {
    ShareState.Malformed -> ProblemText(R.drawable.ic_file, "There was nothing to keep",
        "ThyMind found no link, words or file in what was shared. Try sharing it again from the other app.", OKAY)
    ShareState.TooManyItems -> ProblemText(R.drawable.ic_stack, "That’s more than $MAX_SHARE_ITEMS",
        "You can save up to $MAX_SHARE_ITEMS things at once. Share them in two goes.", OKAY)
    ShareState.FileTooBig -> ProblemText(R.drawable.ic_file, "That file is too big",
        "ThyMind keeps files up to 5 GB. Share a smaller copy or a shorter clip.", OKAY)
    ShareState.PdfHasPassword -> ProblemText(R.drawable.ic_lock, "This PDF has a password",
        "ThyMind can’t open a PDF that needs a password. Save a copy without it and share that one.", OKAY)
    ShareState.NoLogin -> ProblemText(R.drawable.ic_library_profile, "Sign in first",
        "ThyMind keeps things once you’re signed in. Open ThyMind, sign in, then share this again.", "Open ThyMind")
    is ShareState.Ready -> null
}

/**
 * The sheet that says why nothing was saved, titled Nothing was saved. Left untouched it fades away after
 * [PROBLEM_FADE_MS] (share-sheet-fades-by-itself).
 *
 * @param problem The state to explain; [ShareState.Ready] shows nothing.
 * @param onAction Called once the sheet has gone after its button was tapped.
 * @param onClose Called once the sheet has gone any other way.
 */
@Composable
fun ShareProblem(problem: ShareState, onAction: () -> Unit, onClose: () -> Unit) {
    val text = problemText(problem) ?: return
    var touched by remember { mutableStateOf(false) }
    var acted by remember { mutableStateOf(false) }
    ShareFrame(
        NOT_SAVED_TITLE,
        counting = !touched,
        waitMs = PROBLEM_FADE_MS,
        onTouch = { touched = true },
        onClosed = { if (acted) onAction() else onClose() },
    ) { frame ->
        val colors = menuColors()
        Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp).padding(top = 20.dp, bottom = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Box(Modifier.size(64.dp).clip(CircleShape).background(colors.rowText.copy(alpha = 0.1f)), contentAlignment = Alignment.Center) {
                Image(painterResource(text.icon), null, Modifier.size(28.dp), colorFilter = ColorFilter.tint(colors.rowText))
            }
            Spacer(Modifier.height(16.dp))
            Text(text.title, style = TextStyle(color = colors.rowText, fontSize = 20.sp, lineHeight = 26.sp, fontWeight = FontWeight.SemiBold,
                fontFamily = CardMenuFont, textAlign = TextAlign.Center))
            Spacer(Modifier.height(8.dp))
            Text(text.body, Modifier.padding(horizontal = 12.dp), style = TextStyle(color = colors.rowText.copy(alpha = 0.65f), fontSize = 17.sp,
                lineHeight = 24.sp, fontFamily = CardMenuFont, textAlign = TextAlign.Center))
            Spacer(Modifier.height(28.dp))
            ShareDoneButton(text.button, frame::left, { acted = true; frame.close() })
        }
    }
}

private const val OKAY = "Okay"
private const val NOT_SAVED_TITLE = "Nothing was saved"
