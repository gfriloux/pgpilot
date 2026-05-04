# Quickstart

Get your first key up and running in 5 minutes.

## Step 1: Create Your First Key

1. Launch pgpilot
2. Click **Créer** (Create) in the sidebar
3. Fill in the form:
   - **Name**: Your full name (e.g., "Alice Wonderland")
   - **Email**: Your email address (e.g., "alice@example.com")
   - **Subkey Expiry**: Keep the default (keys created by pgpilot do not expire)
   - **Include SSH Auth**: Leave unchecked (you can add it later if needed)
4. Click **Créer clef** (Create Key)
5. pgpilot will call `gpg --batch --generate-key`. This takes 10–30 seconds. You'll see:
   - A progress message
   - Your new key appears in the **My Keys** list on the left
   - A green checkmark "Key created successfully"

**What pgpilot created**:
- A **primary key** (certification-only, ed25519)
- Three **subkeys**:
  - **Sign** subkey (ed25519) — for signing files
  - **Encr** subkey (cv25519) — for decryption
  - **Auth** subkey (ed25519) — for SSH authentication (optional)

---

## Step 2: Export Your Public Key

Share your public key with contacts so they can send you encrypted messages.

1. Select your new key in the **My Keys** list
2. Click the **Export** button (or three-dot menu)
3. Choose one of three options:
   - **File** — saves `YourName.pub.asc` to disk
   - **Clipboard** — copies the key so you can paste it anywhere
   - **Upload** — posts to paste.rs and gives you a shareable link

**Example**: Export to clipboard, then paste into an email or chat.

---

## Step 3: Import a Contact's Key

Receive a public key from someone and add it to your keyring.

### Option A: From a file

1. Click **Importer** (Import) in the sidebar
2. Select **Fichier** (File)
3. Choose the `.asc` file containing their key
4. Click **Importer** — pgpilot will validate and add it

### Option B: From keyserver

1. Click **Importer** (Import)
2. Select **Serveur de clefs** (Keyserver)
3. Enter their email or fingerprint
4. pgpilot queries `keys.openpgp.org` (or another keyserver) and shows the key
5. Click **Importer**

### Option C: From URL or pasted text

1. Click **Importer** (Import)
2. Select **URL** (paste an HTTPS link to a `.asc` file) or **Coller** (paste armored key text)
3. pgpilot fetches or parses the key
4. Click **Importer**

---

## Step 4: Trust Their Key

Once imported, the key appears in **My Keys** with trust level **Undefined**.

To send them encrypted messages, increase their trust:

1. Select their key in the list
2. In the detail panel, find the **Confiance** (Trust) badge
3. Click it and choose:
   - **Marginale** (Marginal) — you've partially verified their identity
   - **Complète** (Full) — you've fully verified their identity
4. Click **Enregistrer** (Save)

**Best practice**: Meet them in person, exchange fingerprints, and verify by hand before trusting.

---

## Step 5: Encrypt a File for Them

Now you can send them encrypted messages.

1. Click **Chiffrer** (Encrypt) in the sidebar
2. Click **Ajouter fichiers** (Add Files) and select a file
3. Check their key in the **Destinataires** (Recipients) list
4. Optionally toggle between `.gpg` (binary) and `.asc` (armored/text) format
5. Click **Chiffrer** (Encrypt)
6. pgpilot creates `yourfile.pdf.gpg` (or `.asc`) in the same folder as the original

Send the encrypted file to them. Only they can decrypt it (using their private key password).

---

## What's Next?

- **Publish your key** — Use **Publier** (Publish) to upload to keyservers so anyone can find your key by email
- **Rotate subkeys** — Use **Renouveler** or **Remplacer** to keep subkeys fresh
- **Backup** — Use **Sauvegarder** to export your private key + revocation certificate
- **YubiKey** — Use **Migrer vers YubiKey** to store your subkeys on a hardware security key

See [Key Management](3-key-management.md) for detailed guides.
