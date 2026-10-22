package com.octaviantocan.mymind.ui

import android.graphics.Bitmap
import android.view.View
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import androidx.core.graphics.Insets
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.octaviantocan.mymind.ui.theme.MymindTheme
import java.nio.ByteBuffer

/** Robolectric qualifiers of the reference phone: 1080 x 2520 px at the 450 dpi override, 384 x 896 dp. */
const val PHONE_QUALIFIERS = "w384dp-h896dp-450dpi"

/** Font scale the reference phone runs at. */
const val PHONE_FONT_SCALE = 1.08f

/** Screen height of the reference phone in pixels. */
const val PHONE_HEIGHT_PX = 2520

/** Status bar height of the reference phone in pixels. */
const val STATUS_BAR_PX = 113

/** Gesture navigation bar height of the reference phone in pixels. */
const val NAV_BAR_PX = 42

/**
 * Sets [content] inside the theme and gives it the reference phone's status bar and gesture navigation insets.
 *
 * @param dark Whether the system is in dark mode.
 * @param content Screen under test.
 */
fun ComposeContentTestRule.setPhoneContent(dark: Boolean, content: @Composable () -> Unit) = setPhoneContent(dark, 0, content)

/**
 * Sets [content] inside the theme with the reference phone's system bar insets and an open keyboard.
 *
 * @param dark Whether the system is in dark mode.
 * @param keyboardPx Height of the keyboard from the bottom of the screen in pixels, or 0 for none.
 * @param content Screen under test.
 */
fun ComposeContentTestRule.setPhoneContent(dark: Boolean, keyboardPx: Int, content: @Composable () -> Unit) {
    // Robolectric's native NIO class cache must initialize before UI and image threads use it concurrently.
    val pixel = Bitmap.createBitmap(1, 1, Bitmap.Config.ARGB_8888)
    try {
        pixel.copyPixelsToBuffer(ByteBuffer.allocate(pixel.byteCount))
    } finally {
        pixel.recycle()
    }
    var view: View? = null
    mainClock.autoAdvance = false
    setContent {
        val local = LocalView.current
        SideEffect { view = local }
        MymindTheme(dark = dark) { content() }
    }
    runOnIdle {
        val insets = WindowInsetsCompat.Builder()
            .setInsets(WindowInsetsCompat.Type.statusBars(), Insets.of(0, STATUS_BAR_PX, 0, 0))
            .setInsets(WindowInsetsCompat.Type.navigationBars(), Insets.of(0, 0, 0, NAV_BAR_PX))
            .setInsets(WindowInsetsCompat.Type.ime(), Insets.of(0, 0, 0, keyboardPx))
            .setVisible(WindowInsetsCompat.Type.ime(), keyboardPx > 0)
            .build()
        ViewCompat.dispatchApplyWindowInsets(checkNotNull(view), insets)
    }
}
