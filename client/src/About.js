import React, { Component } from 'react';
import { connect } from 'react-redux';

import { withStyles } from '@material-ui/core/styles';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import Paper from '@material-ui/core/Paper';


const useStyles = theme => ({
  root: {
    display: 'flex',
    flexWrap: 'wrap',
  },
  paper: {
    width: '680px',
    padding: theme.spacing(3),
  },
  headerBox: {
  	width: '100%',
  },
  img: {
    'max-width': '200px',
    'max-height': '200px',
    width: 'auto',
    height: 'auto',
  }
});

class About extends Component {
  render() {
    const { classes } = this.props;

    return (
      <div className={classes.root}>
        <Paper className={classes.paper}>
          <Box className={classes.headerBox}>
            <Typography variant="h6">What the heck is a fardel?</Typography>
          </Box>
          <br />
          <Typography>
          <strong>A fardel is a small bundle or collection of information
        that you can carry to the decentralized web to share with others.</strong>
          </Typography>
          <br />
          <Typography>
        The contents of a fardel can be a link to a file, an image, or even a 
        short piece of text like the punchline to a joke. It is something others 
        are willing to pay a small price to access (no more than 5 scrt), and 
        because the price is low there is little risk in unpacking one. 
        There are no limitations on what kinds of contents are shared. It can be a
        link to an immutable file on IPFS or a simple Dropbox link. Thus, they
        can be permanent on the blockchain or they can be ephemeral.
        They can be copyable, deletable, sharable.
        You can think of them as anti-NFTs. :P
          </Typography>
          <br />
          <Typography>
        <strong>The Fardels contract 
        creates a social network where users can carry (post), unpack, rate, and 
        comment on fardels created by the community.</strong> 
          </Typography>
          <br />
          <Typography>Ever since the creation 
        of smart contracts there has been a promise
        of building new kinds of decentralized social networks. In practice
        what has been built have been impoverished 
        clones of existing social networks (e.g. EtherTweet) that lack key 
        features, such as the ability to follow others. That is because most 
        smart 
        contract networks do not keep user data private. Plus, they do 
        not
        innovate around using the features of the blockchain to rethink
        the types of relations that the social network aims to foster. With 
        Fardels we aim to improve on both counts. By building on Secret Network,
        important aspects of user data can be kept private. And just as Twitter
        innovated around sharing short pieces of information, Fardels is 
        designed to foster a culture of sharing digital items of small value.
        If the contents are a let down or disappear (for example, it contains a link to a Dropbox file 
        that no longer exists), well, you can write a negative comment and rate
        it down. It didn't cost much so no great loss. But if it was something
        that made you smile, that you found useful, or simply valued in some
        way then you can leave a positive comment and rate it up, so others 
        will know it is something they might want to unpack as well. Fardels with
        a link to an immutable file on IPFS are marked as well.
          </Typography>
          <br />
                <Typography>
        <strong>Can Fardels be deleted?</strong> 
          </Typography>
                    <br />
          <Typography>
        You can seal a fardel making it so no one can unpack it in the future. However, 
        others who have already unpacked a fardel can always view its contents.
        They paid for it after all.
          </Typography>
          <br />

        </Paper>
        <img className={classes.img} src="bundle.png"/>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return {
    keplrEnabled: state.keplrEnabled,
  }
};

const mapDispatchToProps = (dispatch) => {
  return ({
  });
}

export default connect(mapStateToProps, mapDispatchToProps)(withStyles(useStyles)(About));
