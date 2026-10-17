package com.octaviantocan.mymind.ui.library

import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.octaviantocan.mymind.ui.theme.MymindTheme

/**
 * Pull to refresh around a scrolling page, with the indicator in the page's surface and accent colours.
 *
 * @param refreshing True while the refresh runs; the indicator spins until it turns false.
 * @param onRefresh Called when the user pulled far enough and let go.
 * @param modifier Modifier for the box.
 * @param top Distance from the top of the box to the indicator's resting place, under any bar drawn over the page.
 * @param content The scrolling page.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PullRefresh(
    refreshing: Boolean,
    onRefresh: () -> Unit,
    modifier: Modifier = Modifier,
    top: Dp = 0.dp,
    content: @Composable BoxScope.() -> Unit,
) {
    val state = rememberPullToRefreshState()
    PullToRefreshBox(
        isRefreshing = refreshing,
        onRefresh = onRefresh,
        modifier = modifier,
        state = state,
        indicator = {
            PullToRefreshDefaults.Indicator(
                state = state,
                isRefreshing = refreshing,
                modifier = Modifier.align(Alignment.TopCenter).padding(top = top),
                containerColor = MymindTheme.page.surface,
                color = MymindTheme.colors.accent,
            )
        },
        content = content,
    )
}
