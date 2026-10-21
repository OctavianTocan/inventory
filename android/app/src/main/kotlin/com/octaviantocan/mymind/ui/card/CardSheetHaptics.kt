package com.octaviantocan.mymind.ui.card

import android.content.Context
import android.os.Build
import android.os.VibrationAttributes
import android.os.VibrationEffect
import android.os.VibratorManager
import android.provider.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

/** Pully's native click on landing and tick on controls/thresholds, following system touch feedback. */
internal class CardSheetHaptics(private val context: Context) {
    fun land() = feel(VibrationEffect.EFFECT_CLICK)
    fun tick() = feel(VibrationEffect.EFFECT_TICK)

    private fun feel(effect: Int) {
        runCatching {
            if (Settings.System.getInt(context.contentResolver, Settings.System.HAPTIC_FEEDBACK_ENABLED, 1) == 0) return
            val vibrator = context.getSystemService(VibratorManager::class.java)?.defaultVibrator ?: return
            if (!vibrator.hasVibrator()) return
            val vibration = VibrationEffect.createPredefined(effect)
            if (Build.VERSION.SDK_INT >= 33) vibrator.vibrate(vibration, VibrationAttributes.createForUsage(VibrationAttributes.USAGE_TOUCH))
            else vibrator.vibrate(vibration)
        }
    }
}

@Composable
internal fun rememberSheetHaptics(): CardSheetHaptics {
    val context = LocalContext.current.applicationContext
    return remember(context) { CardSheetHaptics(context) }
}
