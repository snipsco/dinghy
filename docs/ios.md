## Getting started - iOS phone


### Dinghy setup

Assuming [rustup](http://rustup.rs) is already installed...

```
cargo install cargo-dinghy

# If it's already installed, add '--force'
cargo install cargo-dinghy --force
```

### iOS phone

On iOS, things are made complicated by the fact that there is no way to run a
naked executable on a device: you need to make it an app, and sign it before
sending it to your phone. Setting up app signature requires some manipulations
that we could not find a way to dispense with through XCode. The good news is,
this is a constant time thing: *you'll have do it once in a while, but it will
cover all your Rust projects*.

Again, we don't need a paying account.

### Creating a signing id

You may skip most of this section if you're already setup to ship code to
your phone.

* You'll need an Apple ID. Chances are you already have one, but you can
    an account there: https://appleid.apple.com/account .
* Go to XCode > Preferences > Accounts, and make sure your Apple ID is listed,
    or add it (`+` bottom well-hidden bottom left).
* View Details (bottom right this time)
* Click the "Create" button in front of "iOS Development" in the top half of
    this windows.

### Creating a certificate

* Plug-in your iPhone (or the device you want to run test on).
* Fire up XCode, and /create a new XCode project/.
* Pick /iOS/, /Single view application/.
* Options box:
    * Make the bundle identifier `some.unique.domainame.Dinghy`.
    * So Product Name is `Dinghy`, and `Organization identifier` is something
        that will be unique and looks like a domain name in reverse.
    * Pick your team.
    * Leave the rest alone
* Save it somewhere.
* You should see the dreadful projects settings screen. The only thing that is
    relevant for us is the "Signing" bit.
    * If you see your Team there, and nothing red or yellows shows up, then
        you're OK. In some cases, you have to push the "repair" button to
        actually obtain a certificate from Apple.

If you're using a Apple ID free account, you will need to come back once a
week to refresh the certificate (aka "repair"). Paying account generate
longer living certificates, so you need this less often.

### Trust the certificate on your phone

On top of project window, make the target "Dinghy>Your Device", and run the
project (play button). XCode may ask you to go to your phone settings
and Trust the certificate. It's in the Preferences > General >
[your dev account name]. It should then start your empty app on the phone.

At this point, we're ready to roll, dinghy should detect XCode and the various
toolchain on its own.

### Try it

Let's try it with dinghy demo project. The project tests with "pass" in the
name are supposed to pass, the one with fail should break.

```
% git clone https://github.com/snipsco/dinghy
% cd dinghy/test-ws
[...]
# these ones should pass
% cargo dinghy -d iphone test pass
[...]
# this one shall not pass
% cargo dinghy -d iphone test fail
[...]
```

### Simulator

There's a [known bug with lldb and the ios
simulator](https://bugs.llvm.org/show_bug.cgi?id=36580) as such, dinghy will
use lldb to attach to the process on macOS to get the exit status from the
simulator.  On Catalina (and probably earlier), this means the user will be
prompted for higher permissions.

### Debugging tips

If you got lost somewhere, here are a few hints to help you make sense of
what is happening. This is more or less what Dinghy use when fishing for
your signing identity.

#### `security find-identity -p codesigning`

Shows you the codesigning identities available where you are. You should see
one or more identities line, made of a long capitalize hex identifier, followed
by a "name". The name is very structured: For iOS development , its starts
with the string "iPhone Developer: ", followed by an email (for an Apple Id
account) or the name of your team. Then comes the developer identifier in
parenthesis.

#### `security find-certificate -a -c "name" -p | openssl x509 -text`

"name" is the identity name (the string between double quotes) from the command
before.

Shows you a certificate that makes the developer a part of a Team.
The certificate is signed and issued by Apple (Issuer:), but the interesting
part is the Subject line: CN= is the same developer name string, and OU= is
the actual Team identifier.

#### Look for provisioning certificates

Last but not least, we need one more certificate that proves that you and
your team has the right to deploy an App (identified by the bundle id we have
chosen while creating the project) on one (or more) devices.

These certificates are in `Library/MobileDevice/Provisioning\ Profiles`.

To read them, you'll need to do

```
security cms -D -i  ~/Library/MobileDevice/Provisioning\ Profiles\....mobileprovision
```

The more interesting keys here are `TeamIdentifier`, `ProvisionedDevices` which
are relatively explicit, and the `Entitlements` dictionary. These entitlements
specify what the certificate is valid for, that is, signing an app identified
by a name.

Dinghy will actually scan this directory to find one that it can use (this is
where the app name being "Dinghy" plays a role).

Phew.
