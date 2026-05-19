fn commitment(member_secret: Field, opening: Field) -> Field {
    member_secret * member_secret + opening
}

fn nullifier(member_secret: Field, forum_domain: Field, post_nonce: Field) -> Field {
    member_secret * forum_domain + post_nonce
}

fn retro_tag(member_secret: Field, slash_domain: Field, post_nonce: Field) -> Field {
    member_secret * slash_domain + post_nonce
}

fn main(
    member_secret: Field,
    opening: Field,
    forum_domain: pub Field,
    slash_domain: pub Field,
    post_nonce: pub Field,
    registered_commitment: pub Field,
    post_nullifier: pub Field,
    post_retro_tag: pub Field,
) {
    assert(registered_commitment == commitment(member_secret, opening));
    assert(post_nullifier == nullifier(member_secret, forum_domain, post_nonce));
    assert(post_retro_tag == retro_tag(member_secret, slash_domain, post_nonce));
}

#[test]
fn accepts_consistent_post_binding() {
    let member_secret: Field = 11;
    let opening: Field = 7;
    let forum_domain: Field = 5;
    let slash_domain: Field = 13;
    let post_nonce: Field = 17;

    let registered_commitment = commitment(member_secret, opening);
    let post_nullifier = nullifier(member_secret, forum_domain, post_nonce);
    let post_retro_tag = retro_tag(member_secret, slash_domain, post_nonce);

    main(
        member_secret,
        opening,
        forum_domain,
        slash_domain,
        post_nonce,
        registered_commitment,
        post_nullifier,
        post_retro_tag,
    );
}

#[test(should_fail)]
fn rejects_bad_nullifier() {
    let member_secret: Field = 11;
    let opening: Field = 7;
    let forum_domain: Field = 5;
    let slash_domain: Field = 13;
    let post_nonce: Field = 17;

    let registered_commitment = commitment(member_secret, opening);
    let bad_nullifier = nullifier(member_secret, forum_domain, post_nonce) + 1;
    let post_retro_tag = retro_tag(member_secret, slash_domain, post_nonce);

    main(
        member_secret,
        opening,
        forum_domain,
        slash_domain,
        post_nonce,
        registered_commitment,
        bad_nullifier,
        post_retro_tag,
    );
}
