package com.octaviantocan.mymind.data

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement

/** JSON configuration shared by every request and response. */
val AppJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
}

@Serializable
data class Tag(val id: String? = null, val name: String? = null, val flags: Int? = null)

@Serializable
data class Cover(
    val src: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val type: String? = null,
    val dominantColor: String? = null,
)

@Serializable
data class Blob(
    val path: String? = null,
    val type: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val dominantColor: String? = null,
)

@Serializable
data class GalleryImage(val src: String? = null, val width: Int? = null, val height: Int? = null)

@Serializable
data class Post(
    val bodyHtml: String? = null,
    val author: String? = null,
    val hasVideo: Boolean? = null,
    val gallery: List<GalleryImage>? = null,
)

@Serializable
data class AppInfo(
    val distributor: String? = null,
    val srcset: String? = null,
    val width: Int? = null,
    val height: Int? = null,
)

@Serializable
data class Product(val price: String? = null, val brand: String? = null)

@Serializable
data class RedditInfo(val title: String? = null, val subreddit: String? = null)

/** A library card. [type] is kept as a string so card types this app does not know still decode. */
@Serializable
data class Card(
    val id: String,
    val type: String,
    val flags: Int,
    val created: String,
    val modified: String,
    val bumped: String,
    val deleted: String? = null,
    val title: String? = null,
    val description: String? = null,
    val url: String? = null,
    val domain: String? = null,
    val siteName: String? = null,
    val author: String? = null,
    val note: String? = null,
    val tags: List<Tag>? = null,
    val spaces: List<String>? = null,
    val pinPosition: Long? = null,
    val cover: Cover? = null,
    val blob: Blob? = null,
    val prose: JsonElement? = null,
    val post: Post? = null,
    val app: AppInfo? = null,
    val product: Product? = null,
    val reddit: RedditInfo? = null,
    val state: String? = null,
)

@Serializable
data class CardPage(val cards: List<Card> = emptyList(), val next: String? = null)

@Serializable
data class Space(
    val id: String,
    val name: String? = null,
    val color: String? = null,
    val position: Long? = null,
    val smart: Boolean? = null,
    val count: Int? = null,
)

@Serializable
data class SpaceList(val spaces: List<Space> = emptyList())

@Serializable
data class TagCount(val name: String, val count: Int? = null)

@Serializable
data class TagList(val tags: List<TagCount> = emptyList())

@Serializable
data class User(val email: String? = null, val kind: String? = null)

@Serializable
data class TokenAnswer(val token: String, val expiresAt: String? = null, val user: User? = null)

@Serializable
data class NonceAnswer(val nonce: String, val expiresAt: String? = null)

@Serializable
data class ErrorAnswer(val error: String? = null)

@Serializable
data class GoogleSignInRequest(val idToken: String, val nonce: String, val device: String)

@Serializable
data class PasswordSignInRequest(val username: String, val password: String, val device: String)
