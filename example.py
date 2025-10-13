from a_sabr_python import AsabrRouter
from a_sabr_python import AsabrBundle



router = AsabrRouter("./tvg.cp", router_type="VolCgrHybridParenting")

src = 4
dst = 8
curr_time = 0.0
excluded_nodes = []

print("== ids ==")
print("id", router.get_node_id("BRITE-PL2 (HEWELIUSZ)"), "for BRITE-PL2 (HEWELIUSZ)")
print("id", router.get_node_id("PERSEUS-M2"), "for PERSEUS-M2")
print("id", router.get_node_id("POPACS 3"), "for POPACS 3")
print("id", router.get_node_id("SINOD-D 1"), "for SINOD-D 1")
print("id", router.get_node_id("gs0"), "for gs0")
print("id", router.get_node_id("gs1"), "for gs1")
print("id", router.get_node_id("gs2"), "for gs2")
print("id", router.get_node_id("gs3"), "for gs3")
print("id", router.get_node_id("gs4"), "for gs4")
print("====")
asabr_bundle = AsabrBundle(
    source=src,
    destinations=[dst],
    priority=0,
    size=10,
    expiration=1000000000000,
)


routing_output = router.route(src, asabr_bundle, curr_time, excluded_nodes)

first_hop_contact = None
for c, dests in routing_output:
    if dst in dests:
        first_hop_contact = c


print(first_hop_contact.contact_id)
print(first_hop_contact.tx_node)
print(first_hop_contact.rx_node)
print(first_hop_contact.start_time)
print(first_hop_contact.end_time)