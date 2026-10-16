package com.octaviantocan.mymind.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import coil3.compose.AsyncImage
import com.octaviantocan.mymind.BuildConfig
import com.octaviantocan.mymind.data.ApiResult
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Repository
import com.octaviantocan.mymind.data.resolveMediaUrl
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch

data class LibraryUiState(
    val cards: List<Card> = emptyList(),
    val loading: Boolean = false,
    val endReached: Boolean = false,
    val error: String? = null,
)

class LibraryViewModel(private val repository: Repository) : ViewModel() {
    private val mutableState = MutableStateFlow(LibraryUiState())
    val state: StateFlow<LibraryUiState> = mutableState
    private var cursor: String? = null

    init {
        loadMore()
    }

    fun loadMore() {
        val current = mutableState.value
        if (current.loading || current.endReached) return
        mutableState.value = current.copy(loading = true, error = null)
        viewModelScope.launch {
            when (val result = repository.cards(cursor)) {
                is ApiResult.Ok -> {
                    cursor = result.value.next
                    mutableState.value = mutableState.value.copy(
                        cards = mutableState.value.cards + result.value.cards,
                        loading = false,
                        endReached = result.value.next == null,
                    )
                }
                is ApiResult.Failed -> mutableState.value =
                    mutableState.value.copy(loading = false, error = describe(result.failure))
            }
        }
    }

    fun signOut() {
        viewModelScope.launch { repository.signOut() }
    }
}

/** Title shown for a card: its title, else its type. */
fun cardHeading(card: Card): String = card.title?.takeIf { it.isNotBlank() } ?: card.type

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LibraryScreen(viewModel: LibraryViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val listState = rememberLazyListState()

    LaunchedEffect(listState) {
        snapshotFlow {
            val last = listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1
            last >= listState.layoutInfo.totalItemsCount - LOAD_AHEAD
        }
            .distinctUntilChanged()
            .filter { it }
            .collect { viewModel.loadMore() }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Library") },
                actions = { TextButton(onClick = viewModel::signOut) { Text("Sign out") } },
            )
        },
    ) { padding ->
        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize().padding(padding),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp),
        ) {
            items(state.cards, key = { it.id }) { CardRow(it) }
            item {
                Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
                    if (state.loading) CircularProgressIndicator()
                    state.error?.let {
                        Text(it, color = MaterialTheme.colorScheme.error)
                        TextButton(onClick = viewModel::loadMore) { Text("Retry") }
                    }
                }
            }
        }
    }
}

@Composable
private fun CardRow(card: Card) {
    val cover = card.cover?.takeIf { it.type != "video" }?.src
    Column(Modifier.fillMaxWidth()) {
        if (cover != null) {
            val ratio = remember(card.id) {
                val w = card.cover?.width
                val h = card.cover?.height
                if (w != null && h != null && w > 0 && h > 0) w.toFloat() / h else 1.5f
            }
            AsyncImage(
                model = resolveMediaUrl(BuildConfig.SERVER_URL, cover),
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxWidth().aspectRatio(ratio.coerceIn(0.5f, 2.5f)),
            )
            Spacer(Modifier.height(6.dp))
        }
        Text(cardHeading(card), style = MaterialTheme.typography.titleMedium)
        Row { card.domain?.let { Text(it, style = MaterialTheme.typography.bodySmall) } }
    }
}

private const val LOAD_AHEAD = 5
