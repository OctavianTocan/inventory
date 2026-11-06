package com.octaviantocan.mymind.ui.moment

import android.content.Context
import android.os.Build
import android.os.VibrationAttributes
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.provider.Settings

/**
 * What the moment's beats feel like: a tick as the star breaks off, clicks as it turns colour, a thud as the card
 * lands, a hit sized by rarity as its face comes round, a tick per star and a click as it lands in the backpack. It
 * follows the phone's touch feedback setting (share-moment-sounds).
 */
internal class MomentHaptics(private val context: Context) {
    private val vibrator: Vibrator? = context.getSystemService(VibratorManager::class.java)?.defaultVibrator

    /** Plays what [cue] feels like, if it has a feel. */
    fun feel(cue: Cue) {
        val effect = when (cue) {
            is Cue.Ting -> if (cue.step == 0) one(VibrationEffect.Composition.PRIMITIVE_TICK, 0.5f) else one(VibrationEffect.Composition.PRIMITIVE_CLICK, 0.4f + 0.2f * cue.step)
            Cue.Thud -> one(VibrationEffect.Composition.PRIMITIVE_THUD, 0.9f, VibrationEffect.EFFECT_HEAVY_CLICK)
            is Cue.Boom -> when {
                cue.power >= 1f -> compose(VibrationEffect.EFFECT_HEAVY_CLICK, VibrationEffect.Composition.PRIMITIVE_CLICK to 1f, VibrationEffect.Composition.PRIMITIVE_THUD to 0.8f)
                cue.power >= 0.6f -> one(VibrationEffect.Composition.PRIMITIVE_CLICK, 0.9f)
                else -> one(VibrationEffect.Composition.PRIMITIVE_CLICK, 0.6f)
            }
            is Cue.Star -> one(VibrationEffect.Composition.PRIMITIVE_TICK, (0.35f + 0.1f * cue.index).coerceAtMost(1f))
            Cue.Pop -> one(VibrationEffect.Composition.PRIMITIVE_CLICK, 0.7f)
            else -> null
        } ?: return
        runCatching {
            if (Settings.System.getInt(context.contentResolver, Settings.System.HAPTIC_FEEDBACK_ENABLED, 1) == 0) return
            val v = vibrator?.takeIf { it.hasVibrator() } ?: return
            if (Build.VERSION.SDK_INT >= 33) v.vibrate(effect, VibrationAttributes.createForUsage(VibrationAttributes.USAGE_TOUCH)) else v.vibrate(effect)
        }
    }

    private fun one(primitive: Int, scale: Float, fallback: Int = if (scale >= 0.6f) VibrationEffect.EFFECT_CLICK else VibrationEffect.EFFECT_TICK) =
        compose(fallback, primitive to scale)

    /** The primitives one after another, or a predefined [fallback] where the motor cannot play them. */
    private fun compose(fallback: Int, vararg parts: Pair<Int, Float>): VibrationEffect? {
        val v = vibrator ?: return null
        if (!v.areAllPrimitivesSupported(*parts.map { it.first }.toIntArray())) return VibrationEffect.createPredefined(fallback)
        return VibrationEffect.startComposition().apply { parts.forEachIndexed { i, (p, s) -> addPrimitive(p, s, if (i == 0) 0 else 40) } }.compose()
    }
}
