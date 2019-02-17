extern crate egg_mode;

mod common;

use common::tokio_core::reactor;
use common::futures::Stream;

use std::collections::HashSet;
use egg_mode::user;
use egg_mode::list;

fn main() {
    let mut core = reactor::Core::new().unwrap();

    let config = common::Config::load(&mut core);
    let handle = core.handle();

    println!("--------------------------------------------------------------------------------");
    println!("Get all follows...");

    let mut follows = HashSet::new();
    core.run(user::friends_ids(config.user_id, &config.token, &handle)
                .map(|r| r.response)
                .for_each(|id| { follows.insert(id); Ok(()) })).unwrap();

    println!("Done!\n");
    println!("Get all list members...");

    let mut f_list = HashSet::new();
    let list_id = list::ListID::ID(1091617707403862017);
    core.run(
        list::members(list_id, &config.token, &handle)
            .map(|r| r.response.id)
            .for_each(|id| { f_list.insert(id); Ok(()) })
        ).unwrap();
    f_list.remove(&858187872);

    println!("Done!\n");

    if follows == f_list {
        println!("No member to add to / remove from list");
    } else {
        // フォローしてるけどリストに入ってないアカウントの取得
        println!("Get follows \\ list...");

        let follow_list = follows.difference(&f_list);
        let follow_list_ids = follow_list.into_iter().collect::<Vec<_>>();

        if follow_list_ids.is_empty() {
            println!("No member to add to list");
        } else {
            println!("Add following accounts to list");
            print_users(follow_list_ids.clone(), &config, &handle, &mut core);

            // コイツらをリストに突っ込む
            // 100件までしか一気に突っ込めない（超えるとエラーが返ってくる）けど，
            // そんなに差分が出ることはまずないと思うので無視．
            // いざとなれば下の split_each でなんとかなる．
            core.run(list::add_member_list(follow_list_ids, list_id, &config.token, &handle)).unwrap();
        }

        // リストに入ってるけどフォローしてないアカウントの取得
        println!("\nGet list \\ follows...");

        let list_follow = f_list.difference(&follows);
        let list_follow_ids = list_follow.into_iter().collect::<Vec<_>>();

        if list_follow_ids.is_empty() {
            println!("No member to remove from list");
        } else {
            println!("Remove following accounts from list");
            print_users(list_follow_ids.clone(), &config, &handle, &mut core);

            // コイツらをリストから外す
            core.run(list::remove_member_list(list_follow_ids, list_id, &config.token, &handle)).unwrap();
        }
    }
}

fn print_users(ids: Vec<&u64>, config: &common::Config, handle: &tokio_core::reactor::Handle, core: &mut tokio_core::reactor::Core) {
    for user in core.run(user::lookup(ids, &config.token, handle)).unwrap() {
        println!("- {} (@{})", user.name, user.screen_name);
    }
}

#[allow(dead_code)]
fn split_each<T>(mut xs: Vec<T>, n: usize) -> Vec<Vec<T>> {
    let mut xss = Vec::new();

    let len = xs.len();
    let t = len / n;

    for _i in 0..t {
        let ys = xs.split_off(n);
        xss.push(xs);
        xs = ys;
    }
    xss.push(xs);

    return xss;
}