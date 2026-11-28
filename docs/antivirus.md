# Antivirus warnings on setup files

A setup file built by Bach Installer is a new, unsigned program that unpacks files, may
register a scheduled task for automatic updates and runs downloaded setup files.
Machine-learning detections in antivirus products (in Microsoft Defender they end
in `!ml`) sometimes flag exactly that behaviour, before any person has judged the
file. This page says what we have seen and what helps.

## What we have seen

Testing Bach Installer on Windows 11 with Microsoft Defender, `Get-MpThreatDetection`
listed detections of the setup files that `scripts/e2e.ps1` builds. They were
`Behavior:Win32/Execution.A!ml`, `Behavior:Win32/Persistence.A!ml` and, earlier,
two other `!ml` names. The same few outputs were flagged each time: the setup
file of the updates test, which has an `[updates]` section and installs a
scheduled task, and the update patch of the patch test. Plain setup files
without those features were not flagged in these runs. The detections name
behaviour, not a known malware family, and none came from `bach.exe` itself.

## What helps

- **Sign the setup file** with a code-signing certificate and a timestamp
  (see [signing](signing.md)). A signed file from a known publisher is
  trusted far more readily, and the signature carries reputation from one
  release to the next.
- **Report the false positive** to the vendor. For Microsoft Defender, submit
  the file at <https://www.microsoft.com/wdsi/filesubmission> as a
  "software developer" and say it is a Bach Installer setup file.
- **Keep the behaviour honest.** Setup runs without administrator rights unless
  `installer.scope = "machine"` or a step says `elevated`, and the update task
  is only registered when you offer automatic updates.
- **On your build machine,** if Defender quarantines test builds, restore the
  file from Windows Security > Protection history, or exclude only the folder
  the tests build into (`%TEMP%\bach-e2e-*`). Bach Installer never changes antivirus
  settings itself.

## In the tests

`scripts/e2e.ps1` checks that every setup file it builds can be opened. If
Windows refuses, it stops with a sentence that names Defender and the folder,
instead of failing later with a confusing error. That is a finding about the
built file, not a failure of the installer under test.

Run the same script on a clean CI runner to compare: a detection that only
appears on one machine is a reputation effect, not a defect in the setup file.
