(function() {
    var type_impls = Object.fromEntries([["vid",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AvidMConfig-for-Keccak256Config\" class=\"impl\"><a class=\"src rightside\" href=\"src/vid/avid_m/config.rs.html#81-95\">Source</a><a href=\"#impl-AvidMConfig-for-Keccak256Config\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"vid/avid_m/config/trait.AvidMConfig.html\" title=\"trait vid::avid_m::config::AvidMConfig\">AvidMConfig</a> for <a class=\"struct\" href=\"vid/avid_m/config/struct.Keccak256Config.html\" title=\"struct vid::avid_m::config::Keccak256Config\">Keccak256Config</a></h3></section></summary><div class=\"impl-items\"><section id=\"associatedtype.BaseField\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/vid/avid_m/config.rs.html#82\">Source</a><a href=\"#associatedtype.BaseField\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.BaseField\" class=\"associatedtype\">BaseField</a> = Fp&lt;MontBackend&lt;FrConfig, 4&gt;, 4&gt;</h4></section><section id=\"associatedtype.Digest\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/vid/avid_m/config.rs.html#84\">Source</a><a href=\"#associatedtype.Digest\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.Digest\" class=\"associatedtype\">Digest</a> = HasherNode&lt;CoreWrapper&lt;Keccak256Core&gt;&gt;</h4></section><section id=\"associatedtype.MerkleTree\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/vid/avid_m/config.rs.html#86\">Source</a><a href=\"#associatedtype.MerkleTree\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.MerkleTree\" class=\"associatedtype\">MerkleTree</a> = MerkleTree&lt;&lt;<a class=\"struct\" href=\"vid/avid_m/config/struct.Keccak256Config.html\" title=\"struct vid::avid_m::config::Keccak256Config\">Keccak256Config</a> as <a class=\"trait\" href=\"vid/avid_m/config/trait.AvidMConfig.html\" title=\"trait vid::avid_m::config::AvidMConfig\">AvidMConfig</a>&gt;::<a class=\"associatedtype\" href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.Digest\" title=\"type vid::avid_m::config::AvidMConfig::Digest\">Digest</a>, HasherDigestAlgorithm, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.86.0/std/primitive.u64.html\">u64</a>, 3, HasherNode&lt;CoreWrapper&lt;Keccak256Core&gt;&gt;&gt;</h4></section><details class=\"toggle method-toggle\" open><summary><section id=\"method.raw_share_digest\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/vid/avid_m/config.rs.html#88-94\">Source</a><a href=\"#method.raw_share_digest\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"vid/avid_m/config/trait.AvidMConfig.html#tymethod.raw_share_digest\" class=\"fn\">raw_share_digest</a>(\n    raw_shares: &amp;[Self::<a class=\"associatedtype\" href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.BaseField\" title=\"type vid::avid_m::config::AvidMConfig::BaseField\">BaseField</a>],\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;Self::<a class=\"associatedtype\" href=\"vid/avid_m/config/trait.AvidMConfig.html#associatedtype.Digest\" title=\"type vid::avid_m::config::AvidMConfig::Digest\">Digest</a>, <a class=\"enum\" href=\"vid/enum.VidError.html\" title=\"enum vid::VidError\">VidError</a>&gt;</h4></section></summary><div class='docblock'>Digest the raw shares into the element type for Merkle tree. <a href=\"vid/avid_m/config/trait.AvidMConfig.html#tymethod.raw_share_digest\">Read more</a></div></details></div></details>","AvidMConfig","vid::avid_m::Config","vid::avid_m::F","vid::avid_m::MerkleTree","vid::avid_m::MerkleProof","vid::avid_m::MerkleCommit"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[3947]}