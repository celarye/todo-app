# Thoerie Open Authentication

## Basisbegrippen

### OAuth 2.0

OAuth 2.0 is een autorisatieprotocol dat applicaties toestaat namens een gebruiker toegang te krijgen tot gegevens zonder het wachtwoord te delen. Het werkt met een systeem van tokens die beperkte toegang verlenen.

**Bron**: [OAuth.net - OAuth 2.0](https://oauth.net/2/)

> _"OAuth 2.0 is the industry-standard protocol for authorization. OAuth 2.0 focuses on client developer simplicity while providing specific authorization flows for web applications, desktop applications, mobile phones, and living room devices."_

### OpenID Connect (OIDC)

OpenID Connect is een identiteitslaag bovenop OAuth 2.0. Terwijl OAuth voor autorisatie zorgt (wat mag je doen?), zorgt OIDC voor authenticatie (wie ben je?). Het voegt functionaliteit toe voor het veilig verifiëren van de identiteit van gebruikers.

**Bron**: [OpenID Connect Foundation](https://openid.net/connect/)

> _"OpenID Connect is a simple identity layer on top of the OAuth 2.0 protocol. It allows Clients to verify the identity of the End-User based on the authentication performed by an Authorization Server, as well as to obtain basic profile information about the End-User in an interoperable and REST-like manner."_

## Token Types

### Access Token

- Een tijdelijke sleutel die toegang geeft tot beveiligde bronnen
- Meestal korte levensduur (minuten tot uren)
- Bevat informatie over welke toegangsrechten de gebruiker heeft

**Bron**: [Auth0 - Tokens](https://auth0.com/docs/secure/tokens/access-tokens)

> _"Access tokens are used in token-based authentication to allow an application to access an API. The application receives an access token after a user successfully authenticates and authorizes access, then passes the access token as a credential when it calls the target API."_

### Bearer Token

- Een type access token dat alleen bezit vereist om te gebruiken
- Wie de token heeft ("bears"), krijgt toegang
- Moet daarom altijd via HTTPS verstuurd worden

**Bron**: [RFC 6750 - OAuth 2.0 Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750)

> _"This specification describes how to use bearer tokens in HTTP requests to access OAuth 2.0 protected resources. Any party in possession of a bearer token (a 'bearer') can use it to get access to the associated resources (without demonstrating possession of a cryptographic key)."_

### Refresh Token

- Lange-termijn token om nieuwe access tokens te verkrijgen
- Heeft langere levensduur dan access tokens
- Stelt applicaties in staat toegang te behouden zonder nieuwe gebruikersinteractie

**Bron**: [OAuth.com - Refresh Tokens](https://www.oauth.com/oauth2-servers/access-tokens/refreshing-access-tokens/)

> _"Refresh tokens are credentials used to obtain access tokens. Refresh tokens are issued to the client by the authorization server and are used to obtain a new access token when the current access token becomes invalid or expires."_

### JWT (JSON Web Token)

- Een gestandaardiseerd formaat voor tokens
- Bevat claims (informatie) in JSON-formaat
- Bestaat uit drie delen: header, payload en handtekening
- Zelfstandig verificeerbaar via digitale handtekening

**Bron**: [JWT.io](https://jwt.io/introduction)

> _"JSON Web Token (JWT) is an open standard (RFC 7519) that defines a compact and self-contained way for securely transmitting information between parties as a JSON object. This information can be verified and trusted because it is digitally signed."_

## OAuth 2.0 Flows

### Authorization Code Flow

**Bron**: [OAuth.net - Authorization Code Flow](https://oauth.net/2/grant-types/authorization-code/)

**Proces**:

1. Client stuurt gebruiker naar authorization server
2. Gebruiker logt in en geeft toestemming
3. Authorization server stuurt een autorisatiecode naar redirect URI
4. Client wisselt code + client secret uit voor tokens
5. Client gebruikt access token voor API-verzoeken

**Voordeel**: Veiligste methode, geschikt voor server-side applicaties

### Implicit Flow

**Bron**: [Auth0 - Implicit Flow](https://auth0.com/docs/get-started/authentication-and-authorization-flow/implicit-flow-with-form-post)

**Proces**:

1. Client stuurt gebruiker naar authorization server
2. Gebruiker logt in en geeft toestemming
3. Access token wordt direct in browser-fragment (#) geretourneerd

**Gebruik**: Legacy flow voor SPA's, nu vaak vervangen door Authorization Code Flow with PKCE **Risico's**: Tokens direct blootgesteld in browser

### Client Credentials Flow

**Bron**: [DigitalOcean - Client Credentials Flow](https://www.digitalocean.com/community/tutorials/an-introduction-to-oauth-2#grant-type-client-credentials)

**Proces**:

1. Client authenticeert direct bij authorization server met client ID en secret
2. Krijgt access token terug

**Gebruik**: Machine-to-machine communicatie, geen gebruikerscontext

### Authorization Code Flow with PKCE

**Bron**: [Auth0 - Authorization Code Flow with PKCE](https://auth0.com/docs/get-started/authentication-and-authorization-flow/authorization-code-flow-with-proof-key-for-code-exchange-pkce)

**Proces**:

1. Client genereert code_verifier en afgeleide code_challenge
2. Client stuurt gebruiker naar authorization server met code_challenge
3. Na gebruikerstoestemming, client ruilt autorisatiecode + code_verifier uit voor tokens

**Voordeel**: Veilig voor publieke clients (SPA's, mobiele apps)

## JWT Token Structuur en Validatie

### JWT Structuur

**Bron**: [JWT.io - Debugger](https://jwt.io/#debugger)

**Onderdelen**: `header.payload.signature`

**Header** bevat algoritme en token type:

```
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "key-id-123"
}
```

**Payload** bevat claims:

```
{
  "sub": "1234567890",
  "name": "John Doe",
  "iat": 1516239022,
  "exp": 1516242622,
  "aud": "client-id-123"
}
```

### JWT Validatie

**Bron**: [Auth0 - JWT Validation](https://auth0.com/docs/secure/tokens/json-web-tokens/validate-json-web-tokens)

**Validatiestappen**:

1. Verificatie van handtekening met publieke sleutel van issuer
2. Controle van `exp` (expiration time)
3. Controle van `iss` (issuer)
4. Controle van `aud` (audience)
5. Controle van `nbf` (not before time), indien aanwezig

## OpenID Connect Details

### ID Token vs Access Token

**Bron**: [Okta - ID Token vs Access Token](https://developer.okta.com/blog/2018/04/10/oauth-authorization-code-grant-type#the-id-token-vs-the-access-token)

**ID Token**:

- JWT dat gebruikersidentiteitsgegevens bevat
- Belangrijkste claims: `sub` (subject identifier), `iss` (issuer), `aud` (audience)
- Bevat gebruikersinfo zoals naam, email, etc.

**Access Token**:

- Voor toegang tot resources (API's)
- Kan JWT zijn, maar niet verplicht
- Bevat scopes voor autorisatie

### UserInfo Endpoint

**Bron**: [OpenID Connect Core - UserInfo Endpoint](https://openid.net/specs/openid-connect-core-1_0.html#UserInfo)

**Doel**: Ophalen van aanvullende gebruikersgegevens **Response voorbeeld**:

```
{
  "sub": "248289761001",
  "name": "Jane Doe",
  "given_name": "Jane",
  "family_name": "Doe",
  "email": "janedoe@example.com",
  "picture": "http://example.com/janedoe/me.jpg"
}
```

## Externe Authenticatie bij Grote Spelers

### Google

**Google Identity Services** **Bron**: [Google Identity - Authentication](https://developers.google.com/identity/gsi/web/guides/overview)

- Biedt "Sign in with Google" functionaliteit
- Gebruikt OAuth 2.0 en OpenID Connect
- **Endpoints**:
    - Authorize: `https://accounts.google.com/o/oauth2/v2/auth`
    - Token: `https://oauth2.googleapis.com/token`
    - UserInfo: `https://openidconnect.googleapis.com/v1/userinfo`
- **Scopes**:
    - `openid` - OIDC basisfunctionaliteit
    - `email`, `profile` - Gebruikersgegevens
    - `https://www.googleapis.com/auth/drive` - Toegang tot Google Drive

**Implementatie**:

1. Registreer je applicatie bij Google Cloud Console
2. Krijg client ID en client secret
3. Implementeer de Google Sign-In button in je applicatie
4. Verwerk de authenticatie respons met tokens

### Facebook

**Facebook Login** **Bron**: [Facebook for Developers - Login](https://developers.facebook.com/docs/facebook-login/) en [Facebook Login - Access Tokens](https://developers.facebook.com/docs/facebook-login/guides/access-tokens)

- Implementeert OAuth 2.0
- **Graph API Versie**: Vereist versiespecificatie in URL's
- Ondersteunt verschillende toegangsniveaus via permissies (`scope`)

**Implementatie**:

1. Maak een app aan op Facebook for Developers
2. Configureer de login functionaliteit
3. Integreer Facebook SDK in je applicatie
4. Gebruik de login button en verwerk de authenticatierespons

### Microsoft

**Microsoft Identity Platform** **Bron**: [Microsoft Identity Platform](https://learn.microsoft.com/en-us/azure/active-directory/develop/) en [Microsoft Identity - Protocols](https://learn.microsoft.com/en-us/azure/active-directory/develop/v2-protocols-oidc)

- Ondersteunt zowel persoonlijke Microsoft-accounts als Microsoft 365 accounts
- Gebaseerd op OAuth 2.0 en OpenID Connect
- **Endpoints**:
    - OAuth met tenant: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`
    - Token: `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token`

**Implementatie**:

1. Registreer je applicatie in Azure Portal
2. Configureer Microsoft Authentication Library (MSAL)
3. Implementeer de inlogflow
4. Verwerk tokens voor toegang tot Microsoft diensten

## Beveiligingsoverwegingen

### Token Storage

**Bron**: [OWASP - Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)

**Access Tokens**:

- Browser memory (variabelen) voor SPA's
- Vermijd localStorage vanwege XSS-risico's

**Refresh Tokens**:

- Httponly, secure cookies voor web applicaties
- Secure storage voor mobiele apps

### Cross-Site Request Forgery (CSRF) Bescherming

**Bron**: [OAuth.net - CSRF](https://oauth.net/articles/authentication/#csrf)

- Gebruik unieke, random state parameter in OAuth requests
- Verifieer state parameter bij callback

### Token Revocation

**Bron**: [OAuth.com - Token Revocation](https://www.oauth.com/oauth2-servers/listing-authorizations/revoking-access/)

- Tokens moeten kunnen worden ingetrokken
- Gebruik `/revoke` endpoint

## Algemene Implementatiestappen

**Bron**: [OAuth.com - Getting Started](https://www.oauth.com/oauth2-servers/getting-started/)

1. **Registratie**: Registreer je applicatie bij de identity provider
2. **Configuratie**: Stel redirect URIs in en definieer gewenste scopes
3. **Integratie**: Implementeer de authenticatieflow in je applicatie
4. **Tokenverwerking**: Verwerk ontvangen tokens en haal gebruikersgegevens op
5. **Beveiliging**: Bewaar refresh tokens veilig en vernieuw access tokens wanneer nodig

Bij correcte implementatie kunnen gebruikers inloggen via een account dat ze al hebben, wat de gebruiksvriendelijkheid verhoogt en je ontlast van het beheren van wachtwoorden.
