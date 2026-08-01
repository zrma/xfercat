# Resolved And Deferred Questions

이번 slice에는 owner decision이 필요한 미결 항목이 없다.

삭제는 non-cascading으로 동작한다. staged plan이 selected profile ID를 참조하면 profile
삭제를 차단하고, active connection만 참조하는 profile은 삭제와 함께 synthetic connection을
해제한다. imported OpenSSH profile은 source config에서 제거한 뒤 refresh한다.
