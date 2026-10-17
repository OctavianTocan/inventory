package com.octaviantocan.mymind.ui.card

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.octaviantocan.mymind.R
import com.octaviantocan.mymind.ui.theme.MymindTheme
import kotlin.math.abs

/** Height of the card header and of its buttons. */
val CARD_HEADER_HEIGHT = 64.dp

private const val HIDDEN_OFFSET = -64f
private const val AT_TOP_POSITION = 50
private const val MIN_SCROLL_STEP = 5
private const val FADE_DISTANCE = 12f
private const val ANIMATION_MS = 200
private const val BACKGROUND_BLUR_DP = 20

/**
 * Where the card header sits for the latest scroll movement.
 *
 * @property offset Vertical shift of the title row in dp; 0 is fully visible, negative moves it up.
 * @property backgroundAlpha Opacity of the header's coloured backdrop, 0 to 1.
 */
@Stable
class HeaderPose internal constructor(val offset: Float, val backgroundAlpha: Float) {
    internal val contentAlpha: Float
        get() = when {
            offset == 0f -> 1f
            offset < -FADE_DISTANCE -> 0f
            else -> 1f - abs(offset) / FADE_DISTANCE
        }
}

/** Tracks the scroll position of a card page and decides whether its header shows. */
@Stable
class CardHeaderState {
    /** The pose the header animates towards. */
    var pose by mutableStateOf(HeaderPose(0f, 0f))
        private set

    private var lastPosition = 0

    /**
     * Feeds the scroll position of the card page.
     *
     * @param position Distance scrolled from the top of the page, in dp.
     */
    fun onScroll(position: Int) {
        val step = position - lastPosition
        pose = when {
            position <= AT_TOP_POSITION -> HeaderPose(0f, 0f)
            abs(step) > MIN_SCROLL_STEP -> if (step < 0) HeaderPose(0f, 1f) else HeaderPose(HIDDEN_OFFSET, 0f)
            else -> pose
        }
        lastPosition = position
    }
}

/**
 * @param background Colour the header sits on.
 * @return White or black, whichever differs more in luminance from [background]; white when they tie.
 */
fun readableOn(background: Color): Color {
    val target = background.luminance()
    val whiteGap = abs(target - Color.White.luminance())
    val blackGap = abs(target - Color.Black.luminance())
    return if (blackGap > whiteGap) Color.Black else Color.White
}

/**
 * The two icons of the card header.
 *
 * @property close Caret on the left button.
 * @property menu Dots on the right button.
 */
@Immutable
class CardHeaderIcons(val close: Painter, val menu: Painter)

/**
 * @return The icons that ship with the app resources.
 */
@Composable
fun defaultCardHeaderIcons() = CardHeaderIcons(
    close = painterResource(R.drawable.ic_caret_down),
    menu = painterResource(R.drawable.ic_horizontal_menu),
)

/**
 * The 64 dp header of the card view: a close caret, the card title and a menu button.
 *
 * @param title Card title shown centred between the buttons; read-only.
 * @param color Colour of the backdrop; the icons and title contrast with it while the backdrop shows.
 * @param state Scroll tracking that decides whether the header is shown.
 * @param onClose Called when the caret is tapped.
 * @param modifier Modifier for the header box.
 * @param page Colour of the page under the header; the icons and title contrast with it while the backdrop is hidden.
 * @param onMenu Called when the menu button is tapped; null leaves the button out.
 * @param icons Pictures of the two buttons.
 */
@Composable
fun CardHeader(
    title: String,
    color: Color,
    state: CardHeaderState,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
    page: Color = color,
    onMenu: (() -> Unit)? = null,
    icons: CardHeaderIcons = defaultCardHeaderIcons(),
) {
    val pose = state.pose
    val spec = tween<Float>(ANIMATION_MS, easing = FastOutSlowInEasing)
    val offset by animateFloatAsState(pose.offset, spec, label = "headerOffset")
    val alpha by animateFloatAsState(pose.contentAlpha, spec, label = "headerAlpha")
    val backgroundAlpha by animateFloatAsState(pose.backgroundAlpha, spec, label = "headerBgAlpha")
    val ink = lerp(readableOn(page), readableOn(color), backgroundAlpha)

    Box(modifier.fillMaxWidth().height(CARD_HEADER_HEIGHT)) {
        if (backgroundAlpha > 0f) {
            Surface(Modifier.fillMaxSize().blur(BACKGROUND_BLUR_DP.dp).alpha(backgroundAlpha), color = color) {}
        }
        Row(
            Modifier
                .fillMaxWidth()
                .height(CARD_HEADER_HEIGHT)
                .graphicsLayer {
                    translationY = offset * density
                    this.alpha = alpha
                },
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            HeaderButton(icons.close, ink, onClose)
            BasicTextField(
                value = title,
                onValueChange = {},
                modifier = Modifier.weight(1f),
                readOnly = true,
                textStyle = TextStyle(
                    color = ink,
                    fontSize = 18.sp,
                    fontFamily = MymindTheme.fonts.nunito,
                    textAlign = TextAlign.Center,
                ),
                singleLine = true,
                cursorBrush = SolidColor(ink),
            )
            if (onMenu != null) HeaderButton(icons.menu, ink, onMenu)
        }
    }
}

@Composable
private fun HeaderButton(icon: Painter, tint: Color, onClick: () -> Unit) {
    Box(
        Modifier
            .size(CARD_HEADER_HEIGHT)
            .clickable(remember { MutableInteractionSource() }, ripple(bounded = false), onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Image(icon, contentDescription = "", colorFilter = ColorFilter.tint(tint))
    }
}
