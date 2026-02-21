use crate::{DoctorReport, ReleaseCheckReport, TrustStatus};

pub fn build_release_check_report(
    trust: TrustStatus,
    doctor: DoctorReport,
    rack_license_audit: String,
) -> ReleaseCheckReport {
    let rack_audit_ok = rack_license_audit == "ok" || rack_license_audit == "not_applicable";
    let overall =
        if trust.default_marketplace_signature_ok && doctor.overall == "ok" && rack_audit_ok {
            "ok"
        } else {
            "needs_attention"
        }
        .to_string();

    let mut recommendations = Vec::new();
    if !trust.default_marketplace_signature_ok {
        recommendations.push("Run `pater trust init` and ensure marketplace.sig is published for default marketplace.".to_string());
    }
    if doctor.overall != "ok" {
        recommendations.push("Run `pater adapter sync --target all` and `pater adapter doctor` until all adapter checks are ok.".to_string());
    }
    if rack_license_audit == "failed" || rack_license_audit == "error" {
        recommendations.push("Run `pater rack license-audit --rack-dir <rack-dir>` and resolve unknown/proprietary plugins before release.".to_string());
    }

    ReleaseCheckReport {
        overall,
        trust,
        doctor,
        rack_license_audit,
        recommendations,
    }
}
