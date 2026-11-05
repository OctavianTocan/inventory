/**
 * Connecting an AI app: consent, then Google, then the `mymind` Worker's own app sign-in decides who gets in
 * (`/app/v1/auth/nonce` and `/app/v1/auth/google`, the gate in `rust/crates/core/src/auth.rs`). This Worker never
 * judges a Google account itself; it only carries the ID token there and keeps the app token it gets back.
 */
import { AuthorizationError, CimdFetchError, type OAuthHelpers, authorizationErrorRedirect } from "@cloudflare/workers-oauth-provider";

import { READ_SCOPE, WRITE_SCOPE, consentPage, errorPage, html, refusedPage } from "./pages";

/** Paths this Worker owns outside `/mcp` itself. Under `/mcp-auth`, never `/mcp/`, so they need no token. */
export const PATHS = {
  authorize: "/mcp-auth/authorize",
  callback: "/mcp-auth/callback",
  token: "/mcp-auth/token",
  register: "/mcp-auth/register",
} as const;

/** What the Worker needs from its environment. */
export interface Env {
  OAUTH_KV: KVNamespace;
  OAUTH_PROVIDER: OAuthHelpers;
  LIBRARY_API: Fetcher;
  LIMITER?: RateLimit;
  PUBLIC_URL: string;
  GOOGLE_CLIENT_ID: string;
  GOOGLE_CLIENT_SECRET: string;
  GOOGLE_AUTH_URL?: string;
  GOOGLE_TOKEN_URL?: string;
}

/** What each grant carries, encrypted by the OAuth library with the token itself. */
export interface Props {
  appToken: string;
  email: string;
  client: string;
}

interface Upstream {
  verifier: string;
  nonce: string;
  client: string;
}

function randomString(bytes: number): string {
  const data = crypto.getRandomValues(new Uint8Array(bytes));
  return btoa(String.fromCharCode(...data)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

async function challenge(verifier: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier));
  return btoa(String.fromCharCode(...new Uint8Array(digest))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function redirect(location: string, headers = new Headers()): Response {
  headers.set("location", location);
  return new Response(null, { status: 302, headers });
}

async function library(env: Env, path: string, body: unknown): Promise<Response> {
  return env.LIBRARY_API.fetch(new URL(path, env.PUBLIC_URL).toString(), {
    method: "POST",
    headers: { "content-type": "application/json", accept: "application/json" },
    body: JSON.stringify(body),
  });
}

async function showConsent(request: Request, env: Env): Promise<Response> {
  const oauth = env.OAUTH_PROVIDER;
  const authRequest = await oauth.parseAuthRequest(request);
  const details = await oauth.describeConsent(authRequest);
  const consent = await oauth.beginConsent(authRequest);
  return html(consentPage(details, consent.handle), 200, consent.headers);
}

async function answerConsent(request: Request, env: Env): Promise<Response> {
  const oauth = env.OAUTH_PROVIDER;
  const form = await request.formData();
  const handle = String(form.get("handle") ?? "");
  if (form.get("decision") !== "approve") {
    const denied = await oauth.denyConsent(request, handle);
    return redirect(denied.redirectTo, denied.headers);
  }
  const scope = form.get("write") ? [READ_SCOPE, WRITE_SCOPE] : [READ_SCOPE];
  const approved = await oauth.approveConsent(request, handle, { scope });
  const client = (await oauth.lookupClient(approved.request.clientId))?.clientName || "an AI app";
  const nonceAnswer = await library(env, "/app/v1/auth/nonce", {});
  if (!nonceAnswer.ok) return html(errorPage("ThyMind did not answer. Try connecting again in a minute."), 502);
  const { nonce } = (await nonceAnswer.json()) as { nonce: string };
  const verifier = randomString(48);
  const { state, headers } = await oauth.beginUpstream(approved.request, {
    data: { verifier, nonce, client } satisfies Upstream,
    headers: approved.headers,
  });
  const google = new URL(env.GOOGLE_AUTH_URL ?? "https://accounts.google.com/o/oauth2/v2/auth");
  google.search = new URLSearchParams({
    client_id: env.GOOGLE_CLIENT_ID,
    redirect_uri: new URL(PATHS.callback, env.PUBLIC_URL).toString(),
    response_type: "code",
    scope: "openid email",
    state,
    nonce,
    code_challenge: await challenge(verifier),
    code_challenge_method: "S256",
    prompt: "select_account",
  }).toString();
  return redirect(google.toString(), headers);
}

async function googleIdToken(env: Env, code: string, verifier: string): Promise<string | undefined> {
  const response = await fetch(env.GOOGLE_TOKEN_URL ?? "https://oauth2.googleapis.com/token", {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded", accept: "application/json" },
    body: new URLSearchParams({
      code,
      client_id: env.GOOGLE_CLIENT_ID,
      client_secret: env.GOOGLE_CLIENT_SECRET,
      redirect_uri: new URL(PATHS.callback, env.PUBLIC_URL).toString(),
      grant_type: "authorization_code",
      code_verifier: verifier,
    }),
  });
  if (!response.ok) return undefined;
  const body = (await response.json()) as { id_token?: string };
  return body.id_token;
}

async function finish(request: Request, env: Env): Promise<Response> {
  const oauth = env.OAUTH_PROVIDER;
  const { request: original, data, headers } = await oauth.finishUpstream<Upstream>(request);
  const params = new URL(request.url).searchParams;
  const code = params.get("code");
  if (params.get("error") || !code) return redirect(authorizationErrorRedirect(original, "access_denied"), headers);
  const idToken = await googleIdToken(env, code, data.verifier);
  if (!idToken) return html(errorPage("Google sign-in could not be finished. Start connecting again from the app."), 502, headers);
  const answer = await library(env, "/app/v1/auth/google", { idToken, nonce: data.nonce, device: `MCP: ${data.client}`.slice(0, 100) });
  if (answer.status === 401) return html(refusedPage(authorizationErrorRedirect(original, "access_denied")), 403, headers);
  if (!answer.ok) return html(errorPage("ThyMind could not check this Google sign-in. Start connecting again from the app."), 502, headers);
  const { token, user } = (await answer.json()) as { token: string; user: { email: string } };
  const { redirectTo } = await oauth.completeAuthorization({
    request: original,
    userId: encodeURIComponent(user.email),
    metadata: { client: data.client },
    scope: original.scope,
    props: { appToken: token, email: user.email, client: data.client } satisfies Props,
  });
  return redirect(redirectTo, headers);
}

/** The Worker's pages outside `/mcp`: the authorize page and Google's callback. Anything else is not found. */
export const signIn: ExportedHandler<Env> = {
  async fetch(request, env) {
    const path = new URL(request.url).pathname;
    try {
      if (path === PATHS.authorize && request.method === "GET") return await showConsent(request, env);
      if (path === PATHS.authorize && request.method === "POST") return await answerConsent(request, env);
      if (path === PATHS.callback && request.method === "GET") return await finish(request, env);
    } catch (error) {
      if (error instanceof AuthorizationError && error.redirectTo) return redirect(error.redirectTo);
      if (error instanceof AuthorizationError) return html(errorPage(`${error.description} Start connecting again from the app.`), 400);
      if (error instanceof CimdFetchError) return html(errorPage("This app could not be checked, so it can't connect."), 400);
      throw error;
    }
    return new Response("Not found", { status: 404, headers: { "content-type": "text/plain; charset=utf-8" } });
  },
};
