package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.theme.MymindTheme

private val CIRCLE = 100.dp
private val DOT = 5.dp

/**
 * The Serendipity tab: one card of the deck at a time with Forget and Keep, and a dot per card of the deck.
 *
 * @param viewModel Library behind the tab.
 * @param onOpenCard Called with the card on show when it is tapped.
 * @param modifier Modifier for the page.
 */
@Composable
fun SerendipityScreen(viewModel: LibraryViewModel, onOpenCard: (Card) -> Unit, modifier: Modifier = Modifier) {
    val state by viewModel.serendipity.state.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { if (state.deck.isEmpty() || state.finished) viewModel.serendipity.deal() }
    SerendipityContent(
        state = state,
        onOpenCard = onOpenCard,
        onKeep = viewModel.serendipity::keep,
        onForget = viewModel::forgetSerendipityCard,
        onDeal = viewModel.serendipity::deal,
        modifier = modifier,
    )
}

/**
 * The Serendipity tab without a view model.
 *
 * @param state Deck and position.
 * @param onOpenCard Called with the card on show when it is tapped.
 * @param onKeep Called when Keep is tapped.
 * @param onForget Called when Forget is tapped.
 * @param onDeal Called to draw a new deck.
 * @param modifier Modifier for the page.
 */
@Composable
fun SerendipityContent(
    state: SerendipityState,
    onOpenCard: (Card) -> Unit,
    onKeep: () -> Unit,
    onForget: () -> Unit,
    onDeal: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(modifier.fillMaxSize().background(SerendipityBackground).clipToBounds()) {
        BoxWithConstraints(Modifier.fillMaxSize().statusBarsPadding()) {
            val height = maxHeight
            when {
                state.error != null && state.deck.isEmpty() -> WebErrorScreen(onRetry = onDeal)
                state.dealt && !state.loading && state.deck.isEmpty() -> DeckMessage(EMPTY_DECK, onDeal)
                state.loading || state.deck.isEmpty() -> PageSpinner()
                state.finished -> DeckMessage(FINISHED_DECK, onDeal)
                else -> {
                    val card = checkNotNull(state.current)
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .height(height * CARD_CENTRE * 2)
                            .padding(horizontal = 30.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(Modifier.fillMaxWidth().heightIn(max = height * CARD_MAX_HEIGHT).clip(RoundedCornerShape(TILE_RADIUS))) {
                            CardTile(card, onClick = { onOpenCard(card) }, deck = true)
                        }
                    }
                    Row(Modifier.align(Alignment.TopCenter).offset(y = height * BUTTONS_CENTRE - CIRCLE / 2)) {
                        DeckButton("Forget", onForget)
                        DeckButton("Keep", onKeep)
                    }
                    Row(
                        Modifier.align(Alignment.TopCenter).offset(y = height * DOTS_TOP).blur(1.dp),
                        horizontalArrangement = Arrangement.spacedBy(DOT),
                    ) {
                        state.deck.indices.forEach { i ->
                            Box(
                                Modifier
                                    .size(DOT)
                                    .alpha(if (i < state.index) 1f else 0.3f)
                                    .background(Color.White, CircleShape),
                            )
                        }
                    }
                }
            }
        }
    }
}

/** A line of white text in the middle of the page; a tap deals again. */
@Composable
private fun DeckMessage(text: String, onDeal: () -> Unit) {
    Box(Modifier.fillMaxSize().clickable(onClick = onDeal), contentAlignment = Alignment.Center) {
        Text(text, style = TextStyle(color = Color.White, fontSize = 16.sp, fontFamily = MymindTheme.fonts.nunito))
    }
}

private const val FINISHED_DECK = "All done. Tap for a new round."
private const val EMPTY_DECK = "Nothing to rediscover right now."

/** One of the two 100 dp round buttons with an outline and a label. */
@Composable
private fun DeckButton(label: String, onClick: () -> Unit) {
    val outline = Color.White.copy(alpha = 0.8f)
    Box(
        Modifier
            .size(CIRCLE)
            .clip(CircleShape)
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Canvas(Modifier.matchParentSize()) {
            val stroke = 2.dp.toPx()
            drawCircle(outline, radius = size.minDimension / 2 - stroke / 2, style = Stroke(stroke))
        }
        Text(label, style = TextStyle(color = Color.White, fontSize = 16.sp, fontFamily = MymindTheme.fonts.nunito))
    }
}

/** A still colour wash standing in for the web page's animated WebGL gradient, from colours sampled off the capture. */
private val SerendipityBackground = Brush.linearGradient(
    0f to Color(0xFFB8B1D5),
    0.3f to Color(0xFF7D81A3),
    0.55f to Color(0xFF2F3E5D),
    0.75f to Color(0xFF050E29),
    1f to Color(0xFF030303),
    start = Offset(Float.POSITIVE_INFINITY, 0f),
    end = Offset(0f, Float.POSITIVE_INFINITY),
)

private const val CARD_CENTRE = 0.374f
private const val CARD_MAX_HEIGHT = 0.577f
private const val BUTTONS_CENTRE = 0.797f
private const val DOTS_TOP = 0.881f
