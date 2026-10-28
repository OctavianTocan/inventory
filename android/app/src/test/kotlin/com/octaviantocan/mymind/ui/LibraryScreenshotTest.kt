package com.octaviantocan.mymind.ui

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.SearchType
import com.octaviantocan.mymind.ui.library.SpaceContent
import com.octaviantocan.mymind.ui.library.SpaceListContent
import com.octaviantocan.mymind.ui.library.SpacesState
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.menu.CardMenuContent
import com.octaviantocan.mymind.ui.menu.CardMenuPanel
import com.octaviantocan.mymind.ui.menu.CardMenuIcons
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.menu.InlineSpaces
import com.octaviantocan.mymind.ui.menu.LocalCardMenuAccent
import com.octaviantocan.mymind.ui.menu.SpaceChoices
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.menu.cardMenuItems
import com.octaviantocan.mymind.ui.menu.defaultCardMenuIcons
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.json.Json
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The library screens at the reference phone's size, in the system mode of the parameter, with the account dark. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class LibraryScreenshotTest(private val dark: Boolean) {
    @get:Rule
    val compose = createComposeRule()

    private val mode = if (dark) "dark" else "light"

    private fun render(content: @Composable () -> Unit) {
        compose.setPhoneContent(dark, content)
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun capture(name: String) = compose.onRoot().captureRoboImage("screenshots/${name}_$mode.png")

    private fun shell(tab: Tab = Tab.Everything, overlay: @Composable () -> Unit = {}, page: @Composable () -> Unit) =
        render {
            ShellFrame(tab, onSelectTab = {}, scheme = PageScheme.Dark, overlay = { overlay() }, page = page)
        }

    @Test
    fun everythingTop() {
        shell { SampleEverything() }
        capture("S06_everything_top")
    }

    @Test
    fun everythingScrolled() {
        val grid = LazyListState()
        shell { SampleEverything(grid = grid) }
        compose.change { runBlocking { grid.scrollToItem(3, 120) } }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        capture("S06_everything_scrolled")
    }

    @Test
    fun everythingSystemScheme() {
        render { ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.System) { SampleEverything() } }
        capture("S06_everything_account_system")
    }

    @Test
    fun everythingLoading() {
        shell { SampleEverything(feed = FeedState(loading = true)) }
        capture("S05_spinner_everything")
    }

    @Test
    fun searchFocused() {
        shell { SampleEverything(searching = true) }
        capture("S09_search_focused")
    }

    @Test
    fun searchResults() {
        shell {
            SampleEverything(searching = true, query = "lorem", type = SearchType.XPost, results = FeedState(LibrarySamples.grid.take(5), endReached = true))
        }
        capture("S09_search_results")
    }

    @Test
    fun spacesTop() {
        shell(Tab.Spaces) { SpaceListContent(LibrarySamples.spaces, onOpenSpace = {}, onRetry = {}) }
        capture("S07_spaces_top")
    }

    @Test
    fun spacesLoading() {
        shell(Tab.Spaces) { SpaceListContent(SpacesState(loading = true), onOpenSpace = {}, onRetry = {}) }
        capture("S05_spinner_spaces")
    }

    @Test
    fun spaceOpen() {
        val space = LibrarySamples.spaces.spaces.first()
        shell(Tab.Spaces) {
            SpaceContent(space, FeedState(LibrarySamples.grid, endReached = true), onBack = {}, onLoadMore = {}, onOpenCard = {}, onAdd = {})
        }
        capture("S07_space_open")
    }

    @Test
    fun cardHeaderRemainsWhileScrolling() {
        val scroll = ScrollState(0)
        shell(overlay = {
            PageTheme(PageScheme.Dark) {
                CardDetail(LibrarySamples.article, onClose = {}, onMenu = {}, scrollState = scroll, headerIcons = com.octaviantocan.mymind.ui.card.defaultCardHeaderIcons())
            }
        }) { SampleEverything() }
        capture("S10_card_rest")
        compose.change { runBlocking { scroll.scrollTo(SCROLL_DOWN_PX) } }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        capture("S10_card_scrolled_down")
        compose.change { runBlocking { scroll.scrollTo(SCROLL_UP_PX) } }
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        capture("S10_card_scrolled_up")
    }

    @Test
    fun fullScreenCardDetail() {
        val scheme = if (dark) PageScheme.Dark else PageScheme.Light
        render {
            ShellFrame(Tab.Everything, onSelectTab = {}, scheme = scheme, overlay = {
                PageTheme(scheme) { CardDetail(LibrarySamples.article, onClose = {}, onMenu = {}) }
            }) { SampleEverything() }
        }
        capture("card_detail")
    }

    @Test
    fun readerMode() {
        val scheme = if (dark) PageScheme.Dark else PageScheme.Light
        val prose = Json.parseToJsonElement(
            """{"type":"doc","content":[""" + (1..8).joinToString(",") {
                """{"type":"paragraph","content":[{"type":"text","text":"A paragraph of the saved page, number $it, long enough to wrap over a few lines of the reader."}]}"""
            } + "]}",
        )
        val article = LibrarySamples.article.copy(prose = prose)
        render {
            ShellFrame(Tab.Everything, onSelectTab = {}, scheme = scheme, overlay = {
                PageTheme(scheme) { CardDetail(article, onClose = {}, onMenu = {}, reading = true) }
            }) { SampleEverything() }
        }
        compose.mainClock.autoAdvance = true
        compose.onNodeWithText("number 8", substring = true).performScrollTo()
        compose.mainClock.autoAdvance = false
        compose.mainClock.advanceTimeBy(SETTLE_MS)
        capture("S15_reader_mode")
    }

    private fun menu(card: Card, name: String, icons: @Composable () -> CardMenuIcons = { defaultCardMenuIcons() }) {
        render {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(
                    Tab.Everything,
                    onSelectTab = {},
                    scheme = PageScheme.Dark,
                    modifier = Modifier.blur(8.dp),
                    overlay = {
                        PageTheme(PageScheme.Dark) {
                            CardDetail(card, onClose = {}, onMenu = {}, headerIcons = com.octaviantocan.mymind.ui.card.defaultCardHeaderIcons())
                        }
                    },
                ) { SampleEverything() }
                CardMenuPanel("Choose one", onDismiss = {}, accent = cardAccent(card)) {
                    val spaces = LibrarySamples.spaces
                    val choices = SpaceChoices(spaces.spaces, selected = setOf("s2"), latest = spaces.latest,
                        suggestion = SpaceSuggestion(spaces.spaces.first(), 3))
                    CardMenuContent(cardMenuItems(card, null), onSelect = {}, icons = icons(),
                        spaces = { InlineSpaces(choices, card, onToggle = {}, onAll = {}) })
                }
            }
        }
        capture(name)
    }

    @Test
    fun cardMenuArticle() = menu(LibrarySamples.article, "S11_card_menu_article")

    @Test
    fun cardMenuNote() = menu(LibrarySamples.note, "S11_card_menu_note")

    @Test
    fun cardMenuVideo() = menu(LibrarySamples.video, "S11_card_menu_video")

    @Test
    fun cardMenuPost() = menu(LibrarySamples.grid[1], "S11_card_menu_post") { defaultCardMenuIcons() }

    @Test
    fun cardMenuRepository() = menu(LibrarySamples.grid[5], "S11_card_menu_repository") { defaultCardMenuIcons() }

    /** Renders the shipped Phosphor glyphs for visual review. */
    @Test
    fun cardMenuShipped() = menu(LibrarySamples.article, "S11_card_menu_real") { defaultCardMenuIcons() }

    companion object {
        private const val SETTLE_MS = 1_500L
        private const val SCROLL_DOWN_PX = 1_200
        private const val SCROLL_UP_PX = 900

        /** System light and system dark. */
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}")
        fun modes() = listOf(arrayOf<Any>(false), arrayOf<Any>(true))
    }
}
